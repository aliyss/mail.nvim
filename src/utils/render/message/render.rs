use crate::utils::buffer::get_real_width;
use crate::utils::render::api;
use std::{collections::HashMap, fmt::Debug};

use comfy_table::{
    Attribute, Cell, ColumnConstraint, Row, Table as ComfyTable, presets::ASCII_FULL_CONDENSED,
};

use nvim_oxi::api::Buffer;
use regex::Regex;

use crate::utils::buffer::render::{FromBuffer, ToBuffer};

pub struct InfoEntry {
    pub key: String,
    pub value: String,
}

pub trait RenderMessage: Debug {
    type Item;

    fn info(&self) -> Vec<InfoEntry>;
    fn body(&self) -> String;
    fn from_data(info: HashMap<String, String>, body: String) -> Self;
}

pub struct Message<T: RenderMessage> {
    pub data: T,
}

impl<T: RenderMessage> Message<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

impl<T: RenderMessage> FromBuffer for Message<T> {
    fn from_buffer(buffer: &Buffer, metadata_offset: Option<usize>) -> anyhow::Result<Self> {
        let line_offset = metadata_offset.unwrap_or(0);
        let lines: Vec<String> = buffer
            .get_lines(line_offset.., true)
            .map_err(|_| anyhow::anyhow!("failed to read lines from buffer"))?
            .map(|nvim_str| nvim_str.to_string())
            .collect();

        let mut info = HashMap::new();
        let mut body_lines = Vec::new();

        let info_re = Regex::new(r"^([^:]+):\s*(.*)$").unwrap();

        for line in lines {
            if let Some(caps) = info_re.captures(&line) {
                let key = caps.get(1).unwrap().as_str().trim().to_string();
                let value = caps.get(2).unwrap().as_str().trim().to_string();
                info.insert(key, value);
            } else {
                body_lines.push(line);
            }
        }

        let body = body_lines.join("\n");

        Ok(Self {
            data: T::from_data(info, body),
        })
    }
}

impl<T: RenderMessage> ToBuffer for Message<T> {
    fn to_buffer(self, buffer: &mut Buffer, line_offset: usize) -> anyhow::Result<Self> {
        let mut lines: Vec<String> = Vec::new();

        let mut table = ComfyTable::new();

        table
            .load_preset(ASCII_FULL_CONDENSED)
            .set_truncation_indicator("…")
            .set_content_arrangement(comfy_table::ContentArrangement::Dynamic);

        let win_width_result = get_real_width(&api::get_current_win(), buffer);

        if let Ok(width) = win_width_result {
            table.set_width(width);
        }

        let info = self.data.info();

        for info_entry in info {
            let mut cells = Vec::new();
            let key_cell = Cell::new(info_entry.key).add_attribute(Attribute::Bold);
            let value_cell = Cell::new(info_entry.value);
            cells.push(key_cell);
            cells.push(value_cell);
            let mut table_row = Row::from(cells);
            table_row.max_height(1);
            table.add_row(table_row);
        }

        match table.column_mut(0) {
            Some(column) => {
                column.set_constraint(ColumnConstraint::ContentWidth);
            }
            None => {
                anyhow::bail!("failed to access key column in the table");
            }
        }

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

        let body = self.data.body();
        if !body.is_empty() {
            lines.push(String::new()); // Add an empty line before the body
            for line in body.lines() {
                lines.push(line.to_string());
            }
        }

        buffer.set_lines(line_offset..line_offset, true, lines)?;

        for (l, s, e) in highlights {
            buffer.add_highlight(0, "Bold", l, s..e)?;
        }

        Ok(self)
    }
}
