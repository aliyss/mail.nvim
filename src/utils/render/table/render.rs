use std::{fmt::Debug, ops::Deref};

use anyhow::Ok;
use nvim_oxi::api::Buffer;

use crate::utils::buffer::render::{FromBuffer, ToBuffer};

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
            // If the line contains the intersection character or is just dashes, skip it.
            if line.contains('┼') || line.chars().all(|c| c == '─' || c == ' ' || c == '┼') {
                line_offset += 1; // Skip the separator line and move to the next line
                continue;
            }

            let cells: Vec<String> = line
                .split('│') // Use char literal
                .map(|cell| cell.trim().to_string())
                .filter(|cell| !cell.is_empty()) // Filter out empty strings from splitting edges
                .collect();

            if cells.is_empty() {
                continue;
            }

            if headers.is_empty() {
                line_offset += 1;
                headers = cells;
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
        let mut data: Vec<Vec<String>> = Vec::new();
        let headers = self.data.headers();
        let has_headers = !headers.is_empty();
        if has_headers {
            self.offset += 1;
            data.push(headers);
        }

        for row in self.data.rows() {
            data.push(row.cells);
        }

        if data.is_empty() {
            return Ok(self);
        }

        let num_columns = data.iter().map(std::vec::Vec::len).max().unwrap_or(0);
        let mut column_widths = vec![0; num_columns];

        for row in &data {
            for (i, cell) in row.iter().enumerate() {
                column_widths[i] = column_widths[i].max(cell.len());
            }
        }

        let mut lines: Vec<String> = Vec::new();
        for (idx, row) in data.into_iter().enumerate() {
            let row_line = row
                .into_iter()
                .enumerate()
                .map(|(i, cell)| format!("{:width$}", cell, width = column_widths[i]))
                .collect::<Vec<_>>()
                .join(" │ ");
            lines.push(row_line);

            if has_headers && idx == 0 {
                let separator = column_widths
                    .iter()
                    .map(|&w| "─".repeat(w))
                    .collect::<Vec<_>>()
                    .join("─┼─");

                self.offset += 1;
                lines.push(separator);
            }
        }

        buffer.set_lines(line_offset..line_offset, true, lines)?;
        Ok(self)
    }
}
