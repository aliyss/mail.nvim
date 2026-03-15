pub mod component;
pub mod config;
pub mod default;
pub mod delete;
pub mod list;
pub mod render;
pub mod reset;
pub mod save;

use std::path::PathBuf;

use nvim_oxi::api::opts::{OptionOpts, OptionScope};
use nvim_oxi::api::{self, Buffer};

use crate::api::account::commands::ListAccounts;
use crate::api::config::Config;
use crate::api::config::ui::view::{UiView, UiViewComponent, UiViewComponentType};
use crate::api::file::TryFile;
use crate::commands::ui::view::render::Table;
use crate::providers::himalaya::HimalayaProvider;

pub(crate) fn render_ui_view(buffer: &mut Buffer, config_path: Option<PathBuf>) {
    let config = Config::read_from_file(config_path).expect("failed to read config file");

    // Determine the file name for the UiView if specified in config, if absolute path use that
    let file_name = Some(&config.default_view_path).map(|path| {
        let actual_path = PathBuf::from(path);
        if actual_path.is_absolute() {
            actual_path
        } else {
            let folder_path = "views/";
            PathBuf::from(folder_path).join(actual_path)
        }
    });

    let opts = OptionOpts::builder().scope(OptionScope::Local).build();
    if let Err(_err) = api::set_option_value("modifiable", true, &opts) {}

    if let Err(_err) = buffer.set_lines(.., false, ["Rendering Mail UI View...", ""]) {}

    let view = UiView::read_from_file_with_path(file_name)
        .expect("failed to read UiView from specified file or default");

    if let Err(_err) = buffer.set_lines(.., false, ["Rendering Mail UI View3...", &view.name]) {}

    // An unbound range on both ends (i.e., `..`) means to replace the whole buffer.
    let json_view_string =
        serde_json::to_string_pretty(&view).expect("failed to serialize UiView to JSON string");
    if let Err(_err) = buffer.set_lines(.., false, [json_view_string]) {}

    // Loop through components and render each
    for component in view.components {
        let lines = vec![format!("Component: {}", component.name)];
        if let Err(_err) = buffer.set_lines(.., false, lines) {}
        match component.component_type {
            UiViewComponentType::Drawer => render_drawer(&component),
            UiViewComponentType::Table => render_table(&component, &config, buffer),
            UiViewComponentType::Detail => render_detail(&component),
            UiViewComponentType::Preview => render_preview(&component),
            UiViewComponentType::File => render_file(&component),
            UiViewComponentType::Other(_) => render_other(&component),
        }
    }

    if let Err(_err) = api::set_option_value("modifiable", false, &opts) {}
}

// Stub implementations for now
fn render_drawer(_component: &UiViewComponent) {
    unimplemented!("Drawer rendering not implemented yet");
}

fn render_table(component: &UiViewComponent, config: &Config, buffer: &mut Buffer) {
    // TODO: Get Provider dynamically
    let provider =
        HimalayaProvider::from_config(config).expect("failed to create provider from config");

    let opts = OptionOpts::builder().scope(OptionScope::Local).build();
    if let Err(_err) = api::set_option_value("modifiable", true, &opts) {}

    // TODO: Make this dynamic based on component configuration
    // Command Group is String can be "accounts", "folders", "emails", etc.
    match component.context.command_group.as_str() {
        "Account" => {
            match component.context.command_type.as_str() {
                "List" => {
                    let accounts = provider.list_accounts().expect("failed to list accounts");
                    let _ = Table::new(accounts).render(buffer);
                } // Handled below
                _ => unimplemented!(
                    "Table rendering for Account action {} not implemented yet",
                    component.context.command_type
                ),
            }
        } // Handled below
        _ => unimplemented!(
            "Table rendering for {} not implemented yet",
            component.context.command_group
        ),
    }

    if let Err(_err) = api::set_option_value("modifiable", false, &opts) {}
}

fn render_detail(_component: &UiViewComponent) {
    unimplemented!("Detail rendering not implemented yet");
}

fn render_preview(_component: &UiViewComponent) {
    unimplemented!("Preview rendering not implemented yet");
}

fn render_file(_component: &UiViewComponent) {
    unimplemented!("File rendering not implemented yet");
}

fn render_other(_component: &UiViewComponent) {
    unimplemented!("Other component rendering not implemented yet");
}
