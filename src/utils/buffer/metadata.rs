use nvim_oxi::api::{
    self, Buffer,
    opts::{OptionOpts, OptionScope},
};

use crate::{
    api::config::ui::view::{UiViewComponent, UiViewComponentType},
    utils::buffer::render::{FromBuffer, ToBuffer},
};

// TODO: Yaml would be more friendly afterwards

#[derive(Debug)]
pub struct BufferMetadata {
    pub component: UiViewComponent,
    pub line_count: usize,
}

impl BufferMetadata {
    #[must_use]
    pub fn new(component: UiViewComponent) -> Self {
        Self {
            component,
            line_count: 0,
        }
    }
}

impl ToBuffer for BufferMetadata {
    fn to_buffer(
        mut self,
        buffer: &mut Buffer,
        line_offset: usize,
    ) -> anyhow::Result<BufferMetadata> {
        let opts = OptionOpts::builder().scope(OptionScope::Local).build();

        let buffer_type = match self.component.component_type {
            UiViewComponentType::Drawer => "drawer",
            UiViewComponentType::Table => "table",
            UiViewComponentType::Detail => "detail",
            UiViewComponentType::Preview => "preview",
            UiViewComponentType::File => "file",
            UiViewComponentType::Other(ref name) => name,
        };

        let group_type = if self.component.context.command_group.is_empty() {
            "unknown"
        } else {
            &self.component.context.command_group.as_str().to_lowercase()
        };

        let file_type = format!("mail-{buffer_type}");
        let component_name = &self.component.name;
        let buffer_name = format!("{component_name}.mail-{group_type}.{buffer_type}");
        api::set_option_value("filetype", file_type, &opts).expect("failed to set buffer filetype");

        let _ = buffer.set_name(&buffer_name);

        match serde_json::to_string_pretty(&self.component) {
            Ok(json_str) => {
                let mut lines = Vec::new();
                lines.push("+++".to_string()); // Using +++ for TOML tradition
                for line in json_str.lines() {
                    lines.push(line.to_string());
                }
                lines.push("+++".to_string());

                self.line_count = lines.len();

                // Insert at the very top (0..0)
                let _ = buffer.set_lines(line_offset..0, true, lines);
            }
            Err(err) => {
                eprintln!("Failed to serialize component to JSON: {err}");
            }
        }

        Ok(self)
    }
}

impl FromBuffer for BufferMetadata {
    fn from_buffer(buffer: &Buffer, _offset: Option<usize>) -> anyhow::Result<Self> {
        let lines: Vec<String> = buffer
            .get_lines(0.., true)
            .map_err(|_| anyhow::anyhow!("failed to read lines from buffer"))?
            .map(|nvim_str| nvim_str.to_string())
            .collect();

        let mut iter = lines.into_iter();

        if iter.next().as_deref() != Some("+++") {
            anyhow::bail!("buffer metadata does not start with expected delimiter");
        }

        let mut json_lines = Vec::new();
        let mut found_end = false;
        let mut line_count = 1;

        for line in iter {
            line_count += 1;
            if line == "+++" {
                found_end = true;
                break;
            }
            json_lines.push(line);
        }

        if !found_end {
            anyhow::bail!("buffer metadata does not contain expected closing delimiter");
        }

        // 3. join("\n") now works because these are standard Strings
        let toml_str = json_lines.join("\n");

        // Since you switched to TOML in the metadata writer, use toml::from_str
        match serde_json::from_str::<UiViewComponent>(&toml_str) {
            Ok(component) => Ok(Self {
                component,
                line_count,
            }),
            Err(err) => {
                eprintln!("Failed to deserialize TOML to UiViewComponent: {err}");
                anyhow::bail!("failed to parse buffer metadata: {err}");
            }
        }
    }
}
