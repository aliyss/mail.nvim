use nvim_oxi::api::Buffer;

pub trait RenderTable {
    fn headers(&self) -> Vec<String>;
    fn rows(&self) -> Vec<RowBuilder>;
}

pub struct RowBuilder {
    cells: Vec<String>,
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
    data: T,
}

impl<T: RenderTable> Table<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }

    pub fn render(self, buffer: &mut Buffer) -> nvim_oxi::Result<()> {
        let mut data: Vec<Vec<String>> = Vec::new();
        let headers = self.data.headers();
        let has_headers = !headers.is_empty();
        if has_headers {
            data.push(headers);
        }

        for row in self.data.rows() {
            data.push(row.cells);
        }

        if data.is_empty() {
            return Ok(());
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
                lines.push(separator);
            }
        }

        buffer.set_lines(0.., true, lines)?;
        Ok(())
    }
}
