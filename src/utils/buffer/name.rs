use nvim_oxi::api::{
    self, Buffer,
    opts::{OptionOpts, OptionScope},
};

use crate::api::config::{
    Config,
    ui::view::{UiViewComponent, UiViewComponentType},
};

// TODO: Yaml would be more friendly afterwards

pub(crate) fn set_command_buffer_metadata(
    buffer: &mut Buffer,
    component: &UiViewComponent,
    _config: &Config,
) {
    let opts = OptionOpts::builder().scope(OptionScope::Local).build();

    api::set_option_value("modifiable", true, &opts).expect("buffer is not modifiable");

    let buffer_type = match component.component_type {
        UiViewComponentType::Drawer => "drawer",
        UiViewComponentType::Table => "table",
        UiViewComponentType::Detail => "detail",
        UiViewComponentType::Preview => "preview",
        UiViewComponentType::File => "file",
        UiViewComponentType::Other(ref name) => name,
    };

    let group_type = if component.context.command_group.is_empty() {
        "unknown"
    } else {
        &component.context.command_group.as_str().to_lowercase()
    };

    let file_type = format!("mail-{buffer_type}");
    let component_name = &component.name;
    let buffer_name = format!("{component_name}.mail-{group_type}.{buffer_type}");
    api::set_option_value("filetype", file_type, &opts).expect("failed to set buffer filetype");

    let _ = buffer.set_name(&buffer_name);

    // 3. Buffer Injection
    // Temporarily allow modification to write the "headers"
    let _ = api::set_option_value("modifiable", true, &opts);

    match serde_json::to_string_pretty(&component) {
        Ok(json_str) => {
            let mut lines = Vec::new();
            lines.push("+++".to_string()); // Using +++ for TOML tradition
            for line in json_str.lines() {
                lines.push(line.to_string());
            }
            lines.push("+++".to_string());

            // Insert at the very top (0..0)
            let _ = buffer.set_lines(0..0, true, lines);
        }
        Err(err) => {
            eprintln!("Failed to serialize component to JSON: {err}");
        }
    }

    // Re-lock the buffer if it's meant to be read-only
    let _ = api::set_option_value("modifiable", false, &opts);
}

#[derive(Debug)]
pub struct CommandBufferData {
    pub component: UiViewComponent,
    pub line_count: usize,
}

pub(crate) fn get_command_buffer_metadata(
    buffer: &mut Buffer,
    _config: &Config,
) -> Result<CommandBufferData, ()> {
    // 1. Get lines and immediately convert them to standard Rust Strings
    let lines: Vec<String> = buffer
        .get_lines(0.., true)
        .map_err(|_| ())?
        .map(|nvim_str| nvim_str.to_string()) // Convert nvim_oxi::String to std::string::String
        .collect();

    let mut iter = lines.into_iter();

    // 2. Now standard methods like as_deref() work on Option<String>
    if iter.next().as_deref() != Some("+++") {
        return Err(());
    }

    let mut json_lines = Vec::new();
    let mut found_end = false;
    let mut line_count = 0;

    for line in iter {
        line_count += 1;
        if line == "+++" {
            found_end = true;
            break;
        }
        json_lines.push(line);
    }

    if !found_end {
        return Err(());
    }

    // 3. join("\n") now works because these are standard Strings
    let toml_str = json_lines.join("\n");

    // Since you switched to TOML in the metadata writer, use toml::from_str
    match serde_json::from_str::<UiViewComponent>(&toml_str) {
        Ok(component) => Ok(CommandBufferData {
            component,
            line_count,
        }),
        Err(err) => {
            eprintln!("Failed to deserialize TOML to UiViewComponent: {err}");
            Err(())
        }
    }
}
