//! Renders list data as `Header: value` details.

use nvim_oxi::api::Buffer;

use crate::api::config::ui::view::UiViewComponent;
use crate::utils::buffer::metadata::BufferMetadata;
use crate::utils::render::ComponentData;
use crate::utils::render::component::{Keymap, render_details};

/// # Errors
///
/// Returns an error if the details fail to render into `buffer`.
pub fn render(
    _component: &UiViewComponent,
    data: ComponentData,
    buffer: &mut Buffer,
    metadata: &BufferMetadata,
) -> anyhow::Result<Vec<Keymap>> {
    let lines: Vec<String> = match data {
        ComponentData::Accounts(accounts) => render_details(&accounts, None),
        ComponentData::Folders(folders) => render_details(&folders, None),
        ComponentData::Emails(emails) => render_details(&emails, None),
        ComponentData::Threads(emails) => render_details(&emails, None),
        ComponentData::EmailMessages(messages) => render_details(&messages, None),
        ComponentData::None => Vec::new(),
    };

    buffer.set_lines(metadata.line_count..metadata.line_count, true, lines)?;

    Ok(Vec::new())
}
