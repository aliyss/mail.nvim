//! Component renderers.
//!
//! Each [`UiViewComponentType`] has a dedicated renderer that knows how to
//! display the [`ComponentData`] produced by
//! [`get_data`](crate::utils::render::get_data). The [`render`] entry point
//! routes the data to the right renderer based on the component's type, so a
//! component can dynamically change which renderer it uses simply by changing
//! its `component_type`.

pub mod detail;
pub mod drawer;
pub mod file;
pub mod other;
pub mod preview;
pub mod table;

use nvim_oxi::api::Buffer;
use nvim_oxi::api::types::Mode;

use crate::api::config::ui::view::{UiViewComponent, UiViewComponentType};
use crate::utils::buffer::metadata::BufferMetadata;
use crate::utils::keymaps::create_localized_keymap;
use crate::utils::render::ComponentData;
use crate::utils::render::table::render::RenderTable;

/// A keymap to be bound in the rendered component's buffer.
pub type Keymap = (Mode, &'static str, String);

/// Keymaps shared by every component regardless of its type.
#[must_use]
pub fn common_keymaps() -> Vec<Keymap> {
    vec![
        (Mode::Normal, "q", ":bdelete<CR>".to_string()),
        (
            Mode::Normal,
            "?",
            ":lua require('mail_nvim').show_help()<CR>".to_string(),
        ),
    ]
}

/// Visual-mode keymaps for the mutating email actions: the visually selected
/// range is added to the multi-select first, so `d`/`m`/`c`/`t`/`a`/`x` act
/// on every selected email instead of only the one under the cursor.
#[must_use]
pub fn email_visual_action_keymaps() -> Vec<Keymap> {
    let select_and = |keys: &'static str, command: &'static str| -> Keymap {
        (
            Mode::Visual,
            keys,
            format!(
                "<Cmd>lua require('mail_nvim').email_select_visual_range()<CR><Esc>:{command}<CR>"
            ),
        )
    };

    vec![
        select_and("d", "MailEmailDelete"),
        select_and("m", "MailEmailMove"),
        select_and("c", "MailEmailCopy"),
        select_and("t", "MailEmailToggleRead"),
        select_and("a", "MailEmailFlagAdd"),
        select_and("x", "MailEmailFlagClear"),
    ]
}

/// Keymaps for the mutating email actions, bound in the email list table and
/// the email file view.
#[must_use]
pub fn email_action_keymaps() -> Vec<Keymap> {
    vec![
        (Mode::Normal, "d", ":MailEmailDelete<CR>".to_string()),
        (Mode::Normal, "m", ":MailEmailMove<CR>".to_string()),
        (Mode::Normal, "c", ":MailEmailCopy<CR>".to_string()),
        (Mode::Normal, "t", ":MailEmailToggleRead<CR>".to_string()),
        (Mode::Normal, "T", ":MailEmailThread<CR>".to_string()),
        (
            Mode::Normal,
            "<C-n>",
            ":MailEmailThreadNext<CR>".to_string(),
        ),
        (
            Mode::Normal,
            "<C-p>",
            ":MailEmailThreadPrevious<CR>".to_string(),
        ),
        (Mode::Normal, "a", ":MailEmailFlagAdd<CR>".to_string()),
        (Mode::Normal, "x", ":MailEmailFlagClear<CR>".to_string()),
    ]
}

/// Routes `data` to the renderer matching `component`'s type and renders it
/// into `buffer`, right after the metadata block (`metadata.line_count`).
///
/// Returns any additional keymaps the component's renderer needs beyond the
/// common ones.
///
/// # Errors
///
/// Returns an error if the renderer fails to write its content to `buffer`.
pub fn render(
    component: &UiViewComponent,
    data: ComponentData,
    buffer: &mut Buffer,
    metadata: &BufferMetadata,
) -> anyhow::Result<Vec<Keymap>> {
    match component.component_type {
        UiViewComponentType::Table => table::render(component, data, buffer, metadata),
        UiViewComponentType::Drawer => drawer::render(component, data, buffer, metadata),
        UiViewComponentType::Detail => detail::render(component, data, buffer, metadata),
        UiViewComponentType::Preview => preview::render(component, data, buffer, metadata),
        UiViewComponentType::File => file::render(component, data, buffer, metadata),
        UiViewComponentType::List => table::render(component, data, buffer, metadata),
        UiViewComponentType::Content => file::render(component, data, buffer, metadata),
        UiViewComponentType::Other(_) => other::render(component, data, buffer, metadata),
    }
}

/// Renders a list-like component as a parseable plain list: a header line
/// followed by one line per item, with cells separated by ` | `.
///
/// Unlike the table renderer there are no borders, but the output stays
/// compatible with the table row parser so selected items can still be
/// re-fetched from the buffer.
pub(crate) fn render_list<T: RenderTable>(data: &T) -> Vec<String> {
    let mut lines = Vec::new();

    let headers = data.headers();
    if !headers.is_empty() {
        lines.push(headers.join(" | "));
    }

    for row in data.rows() {
        lines.push(row.cells.join(" | "));
    }

    lines
}

/// Keymaps that trigger [`ui_enter`](crate::commands::ui::view::navigation::ui_enter)
/// on the rows of a list-like component: the action itself comes from the
/// component's [`enter_action`](UiViewComponent::enter_action), so it can be
/// inferred per component type or overridden in the view file.
pub(crate) fn list_enter_keymaps(
    offset: usize,
    row_count: usize,
    line_count: usize,
    err_msg: &str,
) -> Vec<Keymap> {
    let start_line = line_count + offset + 1;
    let end_line = start_line + row_count;
    let localized_keymap = create_localized_keymap(
        "lua require('mail_nvim').ui_enter()",
        start_line,
        end_line,
        err_msg,
    );

    vec![
        (Mode::Normal, "i", localized_keymap.clone()),
        (Mode::Normal, "<CR>", localized_keymap.clone()),
        (Mode::Normal, "o", localized_keymap),
    ]
}

/// Keymaps that toggle/clear the multi-select of an email list: `<Space>`
/// marks the row under the cursor (and steps down), `u` clears the
/// selection.
pub(crate) fn email_selection_keymaps(
    offset: usize,
    row_count: usize,
    line_count: usize,
    err_msg: &str,
) -> Vec<Keymap> {
    let start_line = line_count + offset + 1;
    let end_line = start_line + row_count;

    let toggle = create_localized_keymap(
        "lua require('mail_nvim').email_toggle_selection()",
        start_line,
        end_line,
        err_msg,
    );
    let clear = create_localized_keymap(
        "lua require('mail_nvim').email_clear_selection()",
        start_line,
        end_line,
        err_msg,
    );

    vec![
        (Mode::Normal, "<Space>", toggle),
        (Mode::Normal, "u", clear),
    ]
}

/// Renders a list-like component as `Header: value` lines.
///
/// When `limit` is set, only the first `limit` rows are rendered.
pub(crate) fn render_details<T: RenderTable>(data: &T, limit: Option<usize>) -> Vec<String> {
    let headers = data.headers();
    let mut lines = Vec::new();

    for (row_index, row) in data.rows().iter().enumerate() {
        if let Some(limit) = limit
            && row_index >= limit
        {
            break;
        }

        for (index, cell) in row.cells.iter().enumerate() {
            let key = headers
                .get(index)
                .map_or_else(|| format!("#{index}"), Clone::clone);
            lines.push(format!("{key}: {cell}"));
        }

        lines.push(String::new());
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::account::Account;

    fn accounts() -> Vec<Account> {
        vec![
            Account::new("nic@aliyssium.com".into(), Some("imap".into()), true),
            Account::new("bob@example.com".into(), None, false),
        ]
    }

    #[test]
    fn render_list_renders_header_and_one_line_per_row() {
        let lines = render_list(&accounts());
        assert_eq!(
            lines,
            vec![
                "Name | Backend | Default".to_string(),
                "nic@aliyssium.com | imap | Yes".to_string(),
                "bob@example.com | None | No".to_string(),
            ]
        );
    }

    #[test]
    fn render_details_renders_header_value_pairs() {
        let lines = render_details(&accounts(), None);
        assert_eq!(
            lines,
            vec![
                "Name: nic@aliyssium.com".to_string(),
                "Backend: imap".to_string(),
                "Default: Yes".to_string(),
                String::new(),
                "Name: bob@example.com".to_string(),
                "Backend: None".to_string(),
                "Default: No".to_string(),
                String::new(),
            ]
        );
    }

    #[test]
    fn render_details_honors_limit() {
        let lines = render_details(&accounts(), Some(1));
        assert_eq!(
            lines,
            vec![
                "Name: nic@aliyssium.com".to_string(),
                "Backend: imap".to_string(),
                "Default: Yes".to_string(),
                String::new(),
            ]
        );
    }
}
