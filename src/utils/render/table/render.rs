use std::{fmt::Debug, ops::Deref};

use comfy_table::{
    Attribute, Cell, ColumnConstraint, Row, Table as ComfyTable, presets::ASCII_MARKDOWN,
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
}

impl Default for RowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RowBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self { cells: Vec::new() }
    }

    #[must_use]
    pub fn with_cell<S: Into<String>>(mut self, cell: S) -> Self {
        self.cells.push(cell.into());
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

            // 2. Split by the pipe '|' character
            let cells: Vec<String> = line
                .split('|')
                .map(|cell| cell.trim().to_string())
                // 3. Filter out empty strings caused by the leading and trailing pipes
                .filter(|cell| !cell.is_empty())
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
                rows.push(RowBuilder { cells });
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

        if let Ok(width) = win_width_result {
            table.set_width(width);
        }

        if has_headers {
            self.offset += 2;
            let mut header_row = Row::from(headers.clone());
            header_row.max_height(1);
            table.set_header(header_row);
        }

        for row in self.data.rows() {
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

        let column = table
            .column_mut(0)
            .expect("table should have at least one column");
        column.set_constraint(ColumnConstraint::ContentWidth);

        let mut lines: Vec<String> = Vec::new();
        let mut highlights = Vec::new(); // Stores (line_idx, start_col, end_col)

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

                    highlights.push((current_line_idx, start_idx, end_idx));

                    clean_line.replace_range(end_idx..end_match.end(), "");
                } else {
                    highlights.push((current_line_idx, start_idx, clean_line.len()));
                    break;
                }
            }
            lines.push(clean_line);
        }

        // Apply to buffer
        buffer.set_lines(line_offset..line_offset, true, lines)?;

        for (l, s, e) in highlights {
            buffer.add_highlight(0, "Bold", l, s..e)?;
        }

        Ok(self)
    }
}
