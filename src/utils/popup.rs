//! Context-sensitive help popups.
//!
//! Pressing `?` in any mail buffer opens a floating window describing the
//! actions available in that context (drawer, table or file view).

use nvim_oxi::Object;
use nvim_oxi::api::opts::{OptionOpts, OptionScope, SetKeymapOpts};
use nvim_oxi::api::types::{
    Mode, WindowAnchor, WindowBorder, WindowConfig, WindowRelativeTo, WindowStyle, WindowTitle,
};
use nvim_oxi::api::{self, Buffer};

use crate::api::config::ui::view::UiViewComponentContextContext;
use crate::utils::buffer::metadata::BufferMetadata;
use crate::utils::buffer::render::FromBuffer;

/// The filetype of help popup buffers, used to identify and close them.
const POPUP_FILE_TYPE: &str = "mail-help";

/// Shows a help popup describing the actions available in the current
/// context. Exported to Lua as `show_help`.
pub fn show_help(_: Object) {
    close_existing_popups();

    let buffer = api::get_current_buf();

    let opts = OptionOpts::builder().buf(buffer.clone()).build();
    let filetype = api::get_option_value::<String>("filetype", &opts).unwrap_or_default();

    let (title, lines) = match filetype.as_str() {
        "mail-drawer" => drawer_help(),
        "mail-table" => table_help(&buffer),
        "mail-file" => file_help(),
        _ => return,
    };

    open_popup(&title, &lines);
}

fn drawer_help() -> (String, Vec<String>) {
    (
        "Mail UI Drawer".to_string(),
        vec![
            "<CR> / o  Expand/collapse or open action".to_string(),
            "J / K     Next / previous sibling".to_string(),
            "<C-n>     First child node".to_string(),
            "<C-p>     Parent node".to_string(),
            "R         Refresh".to_string(),
            "q         Close the drawer".to_string(),
            "?         Show this help".to_string(),
        ],
    )
}

fn table_help(buffer: &Buffer) -> (String, Vec<String>) {
    let (title, action, is_email_list) = match BufferMetadata::from_buffer(buffer, None) {
        Ok(metadata) => {
            let group = metadata.component.context.command_group.as_str();
            let kind = metadata.component.context.command_type.as_str();
            let is_email_list = group == "Email" && kind == "List";
            let action = match (group, kind) {
                ("Account", "List") => "List the folders of the account under the cursor",
                ("Folder", "List") => "List the emails of the folder under the cursor",
                ("Email", "List") => "View the email under the cursor",
                _ => "Open the row under the cursor",
            };
            let context = metadata
                .component
                .context
                .context
                .iter()
                .map(UiViewComponentContextContext::as_str)
                .collect::<Vec<&str>>()
                .join(" / ");
            let title = match (group, kind) {
                ("Account", "List") => "Mail Accounts",
                ("Folder", "List") => "Mail Folders",
                ("Email", "List") => "Mail Emails",
                _ => "Mail Table",
            };
            let title = if context.is_empty() {
                title.to_string()
            } else {
                format!("{title} · {context}")
            };
            (title, action, is_email_list)
        }
        Err(_) => (
            "Mail Table".to_string(),
            "Open the row under the cursor",
            false,
        ),
    };

    let mut lines = vec![format!("i / <CR>  {action}")];

    if is_email_list {
        lines.extend([
            "d          Delete".to_string(),
            "m          Move to folder".to_string(),
            "c          Copy to folder".to_string(),
            "t          Toggle read".to_string(),
            "a          Add flag".to_string(),
            "x          Clear flags".to_string(),
        ]);
    }

    lines.extend([
        "q          Close".to_string(),
        "?          Show this help".to_string(),
    ]);

    (title, lines)
}

fn file_help() -> (String, Vec<String>) {
    (
        "Mail Email".to_string(),
        vec![
            "d         Delete".to_string(),
            "m         Move to folder".to_string(),
            "c         Copy to folder".to_string(),
            "t         Toggle read".to_string(),
            "a         Add flag".to_string(),
            "x         Clear flags".to_string(),
            "q         Close".to_string(),
            "?         Show this help".to_string(),
        ],
    )
}

/// Closes any help popups that are still open.
fn close_existing_popups() {
    let popups: Vec<Buffer> = api::list_bufs()
        .filter(|buffer| {
            let opts = OptionOpts::builder().buf(buffer.clone()).build();
            api::get_option_value::<String>("filetype", &opts)
                .is_ok_and(|filetype| filetype == POPUP_FILE_TYPE)
        })
        .collect();

    for popup in popups {
        let _ = api::command(&format!("bdelete {}", popup.handle()));
    }
}

/// Opens a floating window with `lines` as its content and `title` as its
/// border title.
fn open_popup(title: &str, lines: &[String]) {
    let in_buffer_list = false;
    let is_temporary = true;
    let mut buffer = match api::create_buf(in_buffer_list, is_temporary) {
        Ok(buffer) => buffer,
        Err(err) => {
            nvim_oxi::print!("failed to create popup buffer: {err}");
            return;
        }
    };

    let opts = OptionOpts::builder().buf(buffer.clone()).build();

    if let Err(err) = api::set_option_value("filetype", POPUP_FILE_TYPE, &opts) {
        nvim_oxi::print!("failed to set popup filetype: {err}");
        return;
    }

    if let Err(err) = api::set_option_value("modifiable", true, &opts) {
        nvim_oxi::print!("failed to set popup option value: {err}");
        return;
    }

    if let Err(err) = buffer.set_lines(0..0, true, lines.to_vec()) {
        nvim_oxi::print!("failed to write popup content: {err}");
        return;
    }

    if let Err(err) = api::set_option_value("modifiable", false, &opts) {
        nvim_oxi::print!("failed to set popup option value: {err}");
        return;
    }

    let keymap_opts = SetKeymapOpts::builder().silent(true).build();
    if let Err(err) = buffer.set_keymap(Mode::Normal, "q", ":close<CR>", &keymap_opts) {
        nvim_oxi::print!("failed to set keymap: {err}");
        return;
    }
    if let Err(err) = buffer.set_keymap(Mode::Normal, "<Esc>", ":close<CR>", &keymap_opts) {
        nvim_oxi::print!("failed to set keymap: {err}");
        return;
    }

    // Size the popup to its content and center it on screen.
    let global_opts = OptionOpts::builder().scope(OptionScope::Global).build();
    let columns =
        usize::try_from(api::get_option_value::<i64>("columns", &global_opts).unwrap_or(80))
            .unwrap_or(80);
    let screen_lines =
        usize::try_from(api::get_option_value::<i64>("lines", &global_opts).unwrap_or(24))
            .unwrap_or(24);

    let content_width = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);
    let width = (content_width + 2).clamp(10, columns.saturating_sub(2));
    let height = (lines.len() + 2).clamp(3, screen_lines.saturating_sub(2));

    let mut config = WindowConfig::builder();
    config
        .relative(WindowRelativeTo::Editor)
        .anchor(WindowAnchor::NorthWest)
        .row(u32::try_from(screen_lines.saturating_sub(height) / 2).unwrap_or(0))
        .col(u32::try_from(columns.saturating_sub(width) / 2).unwrap_or(0))
        .width(u32::try_from(width).unwrap_or(u32::MAX))
        .height(u32::try_from(height).unwrap_or(u32::MAX))
        .border(WindowBorder::Rounded)
        .title(WindowTitle::SimpleString(title.into()))
        .style(WindowStyle::Minimal);

    if let Err(err) = api::open_win(&buffer, true, &config.build()) {
        nvim_oxi::print!("failed to open popup: {err}");
    }
}
