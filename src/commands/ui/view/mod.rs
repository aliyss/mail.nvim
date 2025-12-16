use std::path::PathBuf;

use crate::api::{
    config::{
        Config,
        ui::view::{UiView, UiViewComponent, UiViewComponentType},
    },
    file::TryFile,
};
pub mod component;
pub mod config;
pub mod default;
pub mod delete;
pub mod list;
pub mod reset;
pub mod save;

pub(crate) fn render_ui_view(config_path: Option<PathBuf>) {
    let config = Config::read_from_file(config_path).expect("failed to read config file");

    // Determine the file name for the UiView if specified in config, if absolute path use that
    let file_name = config.default_view_path.as_ref().map(|path| {
        let actual_path = PathBuf::from(path);
        if actual_path.is_absolute() {
            actual_path
        } else {
            let folder_path = "views/";
            PathBuf::from(folder_path).join(actual_path)
        }
    });

    let view = UiView::read_from_file(file_name)
        .expect("failed to read UiView from specified file or default");

    // Loop through components and render each
    for component in view.components {
        match component.component_type {
            UiViewComponentType::Drawer => render_drawer(&component),
            UiViewComponentType::Table => render_table(&component),
            UiViewComponentType::Detail => render_detail(&component),
            UiViewComponentType::Preview => render_preview(&component),
            UiViewComponentType::File => render_file(&component),
            UiViewComponentType::Other(_) => render_other(&component),
        }
    }
}

// Stub implementations for now
fn render_drawer(_component: &UiViewComponent) {
    unimplemented!("Drawer rendering not implemented yet");
}

fn render_table(_component: &UiViewComponent) {
    unimplemented!("Table rendering not implemented yet");
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
