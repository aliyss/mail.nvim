use std::{fmt::Debug, ops::Deref};

use comfy_table::{
    Attribute, Cell, ColumnConstraint, Row, Table as ComfyTable, Width, presets::ASCII_MARKDOWN,
};
use nvim_oxi::api::{self, Buffer};
use regex::Regex;

use crate::utils::buffer::{
    get_real_width,
    render::{FromBuffer, ToBuffer},
};

pub trait RenderTable: Deref<Target = [Self::Item]> + Debug {
    type Item;

    fn headers(&self) -> Vec<String>;
    fn rows(&self) -> Vec<RowBuilder>;
    fn from_headers_and_rows(headers: Vec<String>, rows: Vec<RowBuilder>) -> Self;
}

#[derive(Debug, Clone)]
pub struct RowBuilder {
    pub cells: Vec<String>,
    /// An optional highlight group per cell (parallel to `cells`), used to
    /// color-code table rows (e.g. the subject of unread or flagged emails).
    pub styles: Vec<Option<&'static str>>,
}

impl Default for RowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RowBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
            styles: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_cell<S: Into<String>>(mut self, cell: S) -> Self {
        self.cells.push(cell.into());
        self.styles.push(None);
        self
    }

    /// Pushes a cell and highlights it with the given nvim highlight group
    /// (e.g. `MailTableUnread`).
    #[must_use]
    pub fn with_cell_styled<S: Into<String>>(
        mut self,
        cell: S,
        style: Option<&'static str>,
    ) -> Self {
        self.cells.push(cell.into());
        self.styles.push(style);
        self
    }
}

pub struct Table<T: RenderTable> {
    pub data: T,
    pub offset: usize,
}

impl<T: RenderTable> Table<T> {
    pub fn new(data: T) -> Self {
        Self { data, offset: 0 }
    }
}

/// The `ASCII_MARKDOWN` preset draws a `|` at the start, the end and between columns.
const fn borders(columns: usize) -> usize {
    columns + 1
}

/// Sizes the table's columns. A leading `Sel` column is capped at 7 columns
/// and the width left over by the (content-sized) other columns is shared
/// between them, so the selection gutter never dominates the table while it
/// still fills the window.
///
/// The stock `DynamicFullWidth` arrangement cannot do this: once every column
/// has a width, it spreads the leftover space over *all* columns, inflating a
/// constrained `Sel` column right back up.
fn constrain_columns(
    table: &mut ComfyTable,
    headers: &[String],
    rows: &[RowBuilder],
    width: Option<u16>,
) {
    if headers.first().map(String::as_str) != Some("Sel") {
        if let Some(column) = table.column_mut(0) {
            column.set_constraint(ColumnConstraint::ContentWidth);
        }
        return;
    }

    let Some(width) = width else {
        // Without a known table width comfy-table falls back to content-sized
        // columns, which keeps the `Sel` column narrow on its own.
        return;
    };

    // Content width + the 1-column padding comfy-table adds on each side.
    // The widths are measured from the actual cells (not from
    // `column_max_content_widths`, which only accounts for the longest
    // *word* per cell).
    let mut widths: Vec<usize> = headers
        .iter()
        .map(|header| header.chars().count())
        .collect();
    for row in rows {
        for (index, cell) in row.cells.iter().enumerate() {
            if index < widths.len() {
                widths[index] = widths[index].max(cell.chars().count());
            }
        }
    }
    for width in &mut widths {
        *width += 2;
    }
    widths[0] = 7; // `Sel`: enough for the header, a marker and the spinner.

    // A column must never shrink below the width of its header: the rendered
    // table is parsed back by header name (e.g. the `ID` column), so a
    // truncated header would break row lookups. When the window is too
    // narrow for every header, the minimums are capped at the per-column
    // budget instead of overflowing the table.
    let budget = usize::from(width)
        .saturating_sub(borders(widths.len()))
        .saturating_sub(widths[0])
        / widths.len().saturating_sub(1).max(1);
    let minimums: Vec<usize> = headers
        .iter()
        .map(|header| (header.chars().count() + 2).min(budget))
        .collect();

    // Width available to the non-`Sel` columns.
    let available = usize::from(width)
        .saturating_sub(borders(widths.len()))
        .saturating_sub(widths[0]);
    let desired: Vec<usize> = widths.iter().skip(1).copied().collect();

    // Size the non-`Sel` columns like the dynamic arrangement does: columns
    // whose content fits into the current average keep their content width,
    // and the space freed by the small ones is shared by the rest. This keeps
    // the table filling the window without letting long content overflow it.
    let mut remaining = available;
    let mut result: Vec<Option<usize>> = vec![None; desired.len()];
    loop {
        let undecided: Vec<usize> = (0..desired.len())
            .filter(|index| result[*index].is_none())
            .collect();
        if undecided.is_empty() {
            break;
        }
        let average = remaining / undecided.len();
        let mut freed = 0usize;
        let mut changed = false;
        for index in &undecided {
            if desired[*index] <= average {
                result[*index] = Some(desired[*index]);
                freed += desired[*index];
                changed = true;
            }
        }
        if !changed {
            break;
        }
        remaining -= freed;
    }

    let undecided: Vec<usize> = (0..desired.len())
        .filter(|index| result[*index].is_none())
        .collect();
    if undecided.is_empty() {
        // Everything fit: share the leftover over the columns, left to right.
        let share = remaining / desired.len();
        let excess = remaining % desired.len();
        for (index, width) in result.iter_mut().enumerate() {
            *width = Some(width.unwrap_or(0) + share + usize::from(index < excess));
        }
    } else {
        // The columns too wide for their share split the rest equally. The
        // share is fixed up front: re-dividing the shrinking leftover on each
        // step would leak width out of the table (65 / 4 = 16, then 48 / 4 =
        // 12, ...), leaving the table narrower than its pane.
        let share = remaining / undecided.len();
        let excess = remaining % undecided.len();
        for (position, index) in undecided.iter().enumerate() {
            result[*index] = Some(share + usize::from(position < excess));
        }
    }

    for (index, width) in result.into_iter().enumerate() {
        widths[index + 1] = width.unwrap_or(0).max(minimums[index + 1]);
    }

    table.set_content_arrangement(comfy_table::ContentArrangement::Disabled);
    for (index, width) in widths.into_iter().enumerate() {
        if let Some(column) = table.column_mut(index) {
            column.set_constraint(ColumnConstraint::Absolute(Width::Fixed(
                u16::try_from(width).unwrap_or(u16::MAX),
            )));
        }
    }
}

impl<T: RenderTable> FromBuffer for Table<T> {
    fn from_buffer(buffer: &Buffer, metadata_offset: Option<usize>) -> anyhow::Result<Self> {
        let mut line_offset = metadata_offset.unwrap_or(0);
        let lines: Vec<String> = buffer
            .get_lines(line_offset.., true)
            .map_err(|_| anyhow::anyhow!("failed to read lines from buffer"))?
            .map(|nvim_str| nvim_str.to_string())
            .collect();

        let mut rows: Vec<RowBuilder> = Vec::new();
        let mut headers: Vec<String> = Vec::new();

        for line in lines {
            // 1. Skip separator lines (e.g., |-------|-------|)
            // Check if the line is primarily composed of table-structure characters
            if line.contains('|')
                && line
                    .chars()
                    .all(|c| c == '|' || c == '-' || c == '+' || c == ' ')
            {
                line_offset += 1;
                continue;
            }

            // 2. Split by the pipe '|' character. Empty cells (e.g. an empty
            // selection marker or the padding of leading/trailing pipes) are
            // kept so every row has the same number of cells as the header;
            // parsers read their columns by header name.
            let cells: Vec<String> = line
                .split('|')
                .map(|cell| cell.trim().to_string())
                .collect();

            if cells.is_empty() {
                // Only increment if we are still looking for the header
                if headers.is_empty() {
                    line_offset += 1;
                }
                continue;
            }

            if headers.is_empty() {
                headers = cells;
                line_offset += 1;
            } else {
                rows.push(RowBuilder {
                    cells,
                    styles: Vec::new(),
                });
            }
        }

        let table_data = T::from_headers_and_rows(headers, rows);

        Ok(Self {
            data: table_data,
            offset: line_offset,
        })
    }
}

impl<T: RenderTable> ToBuffer for Table<T> {
    fn to_buffer(mut self, buffer: &mut Buffer, line_offset: usize) -> anyhow::Result<Self> {
        let headers = self.data.headers();
        let has_headers = !headers.is_empty();
        let mut table = ComfyTable::new();

        table
            .load_preset(ASCII_MARKDOWN)
            .set_truncation_indicator("…")
            .set_content_arrangement(comfy_table::ContentArrangement::DynamicFullWidth);

        let win_width_result = get_real_width(&api::get_current_win(), buffer);
        let width = win_width_result.ok();

        if let Some(width) = width {
            table.set_width(width);
        }

        if has_headers {
            self.offset += 2;
            let mut header_row = Row::from(headers.clone());
            header_row.max_height(1);
            table.set_header(header_row);
        }

        let rows = self.data.rows();
        for row in &rows {
            let cells = row
                .cells
                .iter()
                .enumerate()
                .map(|(idx, value)| {
                    let mut table_cell = Cell::from(value);
                    if idx == 0 && has_headers {
                        table_cell = table_cell.add_attribute(Attribute::Bold);
                    }
                    table_cell
                })
                .collect::<Vec<Cell>>();
            let mut table_row = Row::from(cells);
            table_row.max_height(1);
            table.add_row(table_row);
        }

        constrain_columns(&mut table, &headers, &rows, width);

        // The highlight group applied to each rendered line: the header gets
        // its own color, then one entry per data row (parallel to `rows`).
        // Separator lines consume no entry.
        let mut row_styles: Vec<Vec<Option<&'static str>>> = Vec::new();
        if has_headers {
            row_styles.push(vec![Some("MailTableHeader"); headers.len()]);
        }
        row_styles.extend(rows.iter().map(|row| row.styles.clone()));
        let mut style_rows = row_styles.into_iter();

        let mut lines: Vec<String> = Vec::new();
        // Stores (line_idx, start_col, end_col)
        let mut highlights = Vec::new();

        let bold_start_re = Regex::new(r"\x1b\[1m").unwrap();
        let bold_reset_re = Regex::new(r"\x1b\[0m").unwrap();

        for (row_idx, raw_row) in table.lines().enumerate() {
            let mut clean_line = raw_row.clone();
            let current_line_idx = line_offset + row_idx;

            while let Some(start_match) = bold_start_re.find(&clean_line) {
                let start_idx = start_match.start();

                clean_line.replace_range(start_idx..start_match.end(), "");

                if let Some(end_match) = bold_reset_re.find(&clean_line) {
                    let end_idx = end_match.start();

                    highlights.push((current_line_idx, "Bold", start_idx, end_idx));

                    clean_line.replace_range(end_idx..end_match.end(), "");
                } else {
                    highlights.push((current_line_idx, "Bold", start_idx, clean_line.len()));
                    break;
                }
            }

            // Separator lines (e.g. `|-------|-------|`) get no styling.
            let is_separator = clean_line.contains('|')
                && clean_line
                    .chars()
                    .all(|c| c == '|' || c == '-' || c == '+' || c == ' ');

            if !is_separator
                && let Some(styles) = style_rows.next()
            {
                let ranges = cell_ranges(&clean_line, styles.len());
                for (cell_idx, style) in styles.into_iter().enumerate() {
                    if let Some(group) = style
                        && let Some((start, end)) = ranges.get(cell_idx)
                    {
                        highlights.push((current_line_idx, group, *start, *end));
                    }
                }
            }

            lines.push(clean_line);
        }

        // Apply to buffer
        buffer.set_lines(line_offset..line_offset, true, lines)?;

        for (l, group, s, e) in highlights {
            buffer.add_highlight(0, group, l, s..e)?;
        }

        Ok(self)
    }
}

/// The byte ranges of each cell of a rendered table line (between the `|`
/// separators of the `ASCII_MARKDOWN` preset).
fn cell_ranges(line: &str, cell_count: usize) -> Vec<(usize, usize)> {
    let pipes: Vec<usize> = line
        .bytes()
        .enumerate()
        .filter_map(|(index, byte)| (byte == b'|').then_some(index))
        .collect();

    (0..cell_count)
        .map(|index| {
            let start = pipes.get(index).map_or(0, |pipe| pipe + 1);
            let end = pipes.get(index + 1).copied().unwrap_or(line.len());
            (start, end)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds the comfy-table the renderer feeds (header + rows, dynamic full
    /// width, sized columns) and returns its rendered header line.
    fn render_table(headers: Vec<&str>, rows: Vec<Vec<&str>>) -> String {
        let mut table = ComfyTable::new();
        table
            .load_preset(ASCII_MARKDOWN)
            .set_content_arrangement(comfy_table::ContentArrangement::DynamicFullWidth)
            .set_width(76);
        table.set_header(Row::from(headers.clone()));
        for row in &rows {
            table.add_row(Row::from(row));
        }
        let headers: Vec<String> = headers.into_iter().map(String::from).collect();
        let rows: Vec<RowBuilder> = rows
            .into_iter()
            .map(|cells| RowBuilder {
                cells: cells.into_iter().map(String::from).collect(),
                styles: Vec::new(),
            })
            .collect();
        constrain_columns(&mut table, &headers, &rows, Some(76));
        table.lines().next().expect("table should render a header")
    }

    /// The width of the first column, measured from the header line.
    fn first_column_width(line: &str) -> usize {
        line.find('|').map_or(0, |first| {
            line[first + 1..].find('|').map_or(0, |second| second)
        })
    }

    #[test]
    fn sel_column_is_capped_at_seven() {
        let header = render_table(
            vec!["Sel", "Name", "Backend", "Default"],
            vec![vec!["", "engelgasse", "IMAP, SMTP", "No"]],
        );
        assert_eq!(first_column_width(&header), 7);
        // The undecided columns still share the surplus, so the table keeps
        // filling the window instead of shrinking to its content width.
        assert_eq!(header.len(), 76);
    }

    #[test]
    fn tables_without_sel_still_fill_the_width() {
        // Message tables have no `Sel` column; they keep the dynamic
        // arrangement and must still span the whole window.
        let header = render_table(
            vec!["Subject", "From"],
            vec![vec!["Invoice", "sender@example.com"]],
        );
        assert_eq!(header.len(), 76);
    }

    #[test]
    fn long_content_does_not_overflow_the_table() {
        // A long subject must not push the table past the window width.
        let header = render_table(
            vec!["Sel", "Subject", "From", "Date"],
            vec![vec!["", "a very long email subject line that keeps going and going", "a-very-long-address@example.com", "2026-08-21 12:34"]],
        );
        assert_eq!(first_column_width(&header), 7);
        assert!(header.len() <= 76, "table overflows: {}", header.len());
    }

    #[test]
    fn cell_ranges_split_on_pipes() {
        let ranges = cell_ranges("| Sel | Subject |", 2);
        assert_eq!(ranges, vec![(1, 6), (7, 16)]);
    }

    #[test]
    fn cell_ranges_handle_missing_pipes() {
        // A truncated line still yields a range per cell (clamped to the
        // end of the line).
        let ranges = cell_ranges("| Sel", 2);
        assert_eq!(ranges, vec![(1, 5), (0, 5)]);
    }

    #[test]
    fn narrow_table_split_does_not_leak_width() {
        // Four columns too wide for their share split the remaining space.
        // The share must be computed up front: re-dividing the shrinking
        // leftover would leak width out of the table, leaving it narrower
        // than its window (e.g. 57 instead of 78 columns).
        let header = render_table(
            vec!["Sel", "Subject", "Sender", "Recipient", "Date"],
            vec![vec![
                "",
                "subject that is very long and exceeds the share",
                "sender-address-that-is-long@example.com",
                "recipient-address-that-is-long@example.com",
                "2026-08-21T12:34:00+00:00",
            ]],
        );
        assert_eq!(first_column_width(&header), 7);
        assert_eq!(
            header.len(),
            76,
            "the table must fill its window exactly, got: {}",
            header.len()
        );
    }
}
