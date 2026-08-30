//! Renders list data as a plain, narrow list (one line per item).
//!
//! The output stays parseable by the table row parser so that selecting an
//! item can resolve its context (e.g. the account a folder list belongs to).

use nvim_oxi::api::Buffer;
use nvim_oxi::api::types::Mode;

use crate::api::config::ui::view::UiViewComponent;
use crate::utils::buffer::metadata::BufferMetadata;
use crate::utils::render::ComponentData;
use crate::utils::render::component::{Keymap, list_enter_keymaps, render_list};
use crate::utils::render::table::render::RenderTable;

/// # Errors
///
/// Returns an error if the list fails to render into `buffer`.
pub fn render(
    _component: &UiViewComponent,
    data: ComponentData,
    buffer: &mut Buffer,
    metadata: &BufferMetadata,
) -> anyhow::Result<Vec<Keymap>> {
    let mut keymaps = Vec::new();

    match data {
        ComponentData::Accounts(accounts) => {
            let lines = render_list(&accounts);
            let offset = usize::from(!accounts.headers().is_empty());
            let row_count = lines.len() - offset;

            buffer.set_lines(metadata.line_count..metadata.line_count, true, lines)?;

            keymaps.extend(list_enter_keymaps(
                offset,
                row_count,
                metadata.line_count,
                "No account selected",
            ));
        }
        ComponentData::Folders(folders) => {
            let lines = render_list(&folders);
            let offset = usize::from(!folders.headers().is_empty());
            let row_count = lines.len() - offset;

            buffer.set_lines(metadata.line_count..metadata.line_count, true, lines)?;

            keymaps.extend(list_enter_keymaps(
                offset,
                row_count,
                metadata.line_count,
                "No folder selected",
            ));
        }
        ComponentData::Emails(emails) => {
            let lines = render_list(&emails);
            let offset = usize::from(!emails.headers().is_empty());
            let row_count = lines.len() - offset;

            buffer.set_lines(metadata.line_count..metadata.line_count, true, lines)?;

            keymaps.extend(list_enter_keymaps(
                offset,
                row_count,
                metadata.line_count,
                "No email selected",
            ));
            keymaps.extend([
                (
                    Mode::Normal,
                    "<C-f>",
                    ":lua require('mail_nvim').email_list_page(1)<CR>".to_string(),
                ),
                (
                    Mode::Normal,
                    "<C-b>",
                    ":lua require('mail_nvim').email_list_page(-1)<CR>".to_string(),
                ),
            ]);
        }
        ComponentData::EmailMessages(messages) => {
            let lines = render_list(&messages);
            buffer.set_lines(metadata.line_count..metadata.line_count, true, lines)?;
        }
        ComponentData::Threads(emails) => {
            let lines = render_list(&emails);
            let offset = usize::from(!emails.headers().is_empty());
            let row_count = lines.len() - offset;

            buffer.set_lines(metadata.line_count..metadata.line_count, true, lines)?;

            keymaps.extend(list_enter_keymaps(
                offset,
                row_count,
                metadata.line_count,
                "No email selected",
            ));
            keymaps.extend([
                (
                    Mode::Normal,
                    "<C-f>",
                    ":lua require('mail_nvim').email_list_page(1)<CR>".to_string(),
                ),
                (
                    Mode::Normal,
                    "<C-b>",
                    ":lua require('mail_nvim').email_list_page(-1)<CR>".to_string(),
                ),
            ]);
        }
        ComponentData::None => {
            nvim_oxi::print!("None rendering not implemented yet.");
        }
    }

    Ok(keymaps)
}
