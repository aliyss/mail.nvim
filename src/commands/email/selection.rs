//! Multi-select for email lists.
//!
//! `<Space>` toggles the row under the cursor and steps down so several rows
//! can be selected quickly; `u` clears the selection. The mutating email
//! commands (delete, move, copy, flags, ...) apply to the selected emails,
//! falling back to the row under the cursor when nothing is selected.

use nvim_oxi::Object;
use nvim_oxi::api::opts::{OptionOpts, OptionScope};
use nvim_oxi::api::{self, Buffer};

use crate::api::email::{Email, ThreadedEmail};
use crate::utils::buffer::metadata::BufferMetadata;
use crate::utils::buffer::render::FromBuffer;
use crate::utils::render::table::marked::HasId;
use crate::utils::render::table::render::{RenderTable, Table};
use crate::utils::selection;

/// Toggles the email under the cursor in the current email list/thread and
/// moves the cursor down, so `<Space>` can select multiple rows quickly.
/// Exported to Lua as `email_toggle_selection`.
pub fn email_toggle_selection(_: Object) {
    let buffer = api::get_current_buf();
    let Some(metadata) = email_list_metadata(&buffer) else {
        return;
    };

    match metadata.component.context.command_type.as_str() {
        "List" => toggle_row::<Vec<Email>>(&buffer, &metadata),
        "Thread" => toggle_row::<Vec<ThreadedEmail>>(&buffer, &metadata),
        _ => {}
    }
}

/// Clears the selection of the current email list/thread and re-renders it
/// without the markers. Exported to Lua as `email_clear_selection`.
pub fn email_clear_selection(_: Object) {
    let buffer = api::get_current_buf();
    let Some(metadata) = email_list_metadata(&buffer) else {
        return;
    };

    selection::clear(&buffer);

    match metadata.component.context.command_type.as_str() {
        "List" => clear_rows::<Vec<Email>>(&buffer, &metadata),
        "Thread" => clear_rows::<Vec<ThreadedEmail>>(&buffer, &metadata),
        _ => {}
    }
}

/// Marks every row of the current visual selection as selected and re-renders
/// the marker column, so the mutating email actions (delete, move, copy,
/// flags, ...) run on the whole range at once. Exported to Lua as
/// `email_select_visual_range`.
///
/// Visual mode itself is left by the calling keymap; this only records the
/// range into the persistent selection.
pub fn email_select_visual_range(_: Object) {
    let buffer = api::get_current_buf();
    let Some(metadata) = email_list_metadata(&buffer) else {
        return;
    };

    let (Ok((start, _)), Ok((end, _))) = (buffer.get_mark('<'), buffer.get_mark('>')) else {
        return;
    };

    // Marks are (1, 0)-indexed lines; the selection can go either direction.
    let (start, end) = (start.min(end), start.max(end));

    match metadata.component.context.command_type.as_str() {
        "List" => select_range::<Vec<Email>>(&buffer, &metadata, start, end),
        "Thread" => select_range::<Vec<ThreadedEmail>>(&buffer, &metadata, start, end),
        _ => {}
    }
}

/// The buffer metadata of the current buffer when it is an email list/thread.
fn email_list_metadata(buffer: &Buffer) -> Option<BufferMetadata> {
    let metadata = BufferMetadata::from_buffer(buffer, None).ok()?;
    if metadata.component.context.command_group == "Email"
        && matches!(
            metadata.component.context.command_type.as_str(),
            "List" | "Thread"
        )
    {
        Some(metadata)
    } else {
        None
    }
}

/// Toggles the row under the cursor and re-renders the marker column in
/// place.
fn toggle_row<T>(buffer: &Buffer, metadata: &BufferMetadata)
where
    T: RenderTable,
    T::Item: HasId,
{
    let Ok(table) = Table::<T>::from_buffer(buffer, Some(metadata.line_count)) else {
        return;
    };
    let row = api::get_current_win().get_cursor().map_or(1, |(r, _)| r);
    let Some(index) = row.checked_sub(table.offset + 1) else {
        return;
    };
    let Some(item) = table.data.get(index) else {
        return;
    };

    let _ = selection::toggle(buffer, item.id());

    // Update the selection marker in place; the metadata block and the
    // keymaps stay untouched, so the cursor keeps its place.
    let _ = render_marked::<T>(buffer, metadata);

    // Step down so `<Space>` keeps selecting the next row.
    let last = buffer.line_count().unwrap_or(row + 1);
    let _ = api::get_current_win().set_cursor(row.saturating_add(1).min(last), 0);
}

/// Re-renders the (now empty) selection of the current list.
fn clear_rows<T>(buffer: &Buffer, metadata: &BufferMetadata)
where
    T: RenderTable,
    T::Item: HasId,
{
    let _ = render_marked::<T>(buffer, metadata);
}

/// Marks every row between `start` and `end` (1-indexed buffer lines) as
/// selected and re-renders the marker column once.
fn select_range<T>(buffer: &Buffer, metadata: &BufferMetadata, start: usize, end: usize)
where
    T: RenderTable,
    T::Item: HasId,
{
    let Ok(table) = Table::<T>::from_buffer(buffer, Some(metadata.line_count)) else {
        return;
    };

    let mut selected = false;
    for line in start..=end {
        // A row at buffer line `line` is the `index`-th item of the table
        // (metadata block and the header sit above the data rows).
        let Some(index) = line.checked_sub(table.offset + 1) else {
            continue;
        };
        let Some(item) = table.data.get(index) else {
            continue;
        };

        let _ = selection::select(buffer, item.id());
        selected = true;
    }

    if selected {
        let _ = render_marked::<T>(buffer, metadata);
    }
}

/// Updates the selection markers of `buffer`'s table in place, without
/// restructuring the buffer.
///
/// The metadata block above the table is collapsed behind a syntax fold
/// (`+++ ... +++`); clearing and rewriting the table body made Neovim
/// recompute that fold over the new content, which could fold list rows (or
/// show the fold text mid-table). Rewriting the `Sel` cell of each row keeps
/// the line structure untouched, so the fold is never disturbed.
fn render_marked<T>(buffer: &Buffer, metadata: &BufferMetadata) -> anyhow::Result<()>
where
    T: RenderTable,
    T::Item: HasId,
{
    let Ok(table) = Table::<T>::from_buffer(buffer, Some(metadata.line_count)) else {
        return Ok(());
    };

    let selected = selection::selected_ids(buffer);
    let mut buffer = buffer.clone();

    // The buffer is non-modifiable once rendered; flip the option back for
    // the rewrite and restore it afterwards.
    let opts = OptionOpts::builder()
        .scope(OptionScope::Local)
        .buf(buffer.clone())
        .build();
    api::set_option_value("modifiable", true, &opts)?;

    let result = (|| -> anyhow::Result<()> {
        for (index, item) in table.data.iter().enumerate() {
            let desired = if selected.contains(item.id()) { '>' } else { ' ' };
            patch_marker(&mut buffer, table.offset + 1 + index, desired)?;
        }
        Ok(())
    })();

    api::set_option_value("modifiable", false, &opts)?;
    result
}

/// Sets the `Sel` cell of the data row at 1-indexed `line` to `marker` (or
/// blanks it out), writing the line back only when it actually changes.
fn patch_marker(buffer: &mut Buffer, line: usize, marker: char) -> anyhow::Result<()> {
    let Some(line_text) = buffer
        .get_lines(line - 1..line, true)?
        .next()
        .map(|nvim_str| nvim_str.to_string())
    else {
        return Ok(());
    };

    let Some(first_pipe) = line_text.find('|') else {
        return Ok(());
    };
    let Some(second_pipe) = line_text[first_pipe + 1..].find('|') else {
        return Ok(());
    };
    let second_pipe = first_pipe + 1 + second_pipe;

    // The row already shows the marker: nothing to write.
    if line_text.as_bytes().get(first_pipe + 2) == Some(&(marker as u8)) {
        return Ok(());
    }

    let mut patched = line_text;
    let inner_len = second_pipe - first_pipe - 1;
    if inner_len < 2 {
        return Ok(());
    }
    let mut inner = vec![b' '; inner_len];
    inner[0] = b' ';
    inner[1] = marker as u8;
    patched.replace_range(first_pipe + 1..second_pipe, &String::from_utf8_lossy(&inner));

    buffer.set_lines(line - 1..line, false, [patched])?;
    Ok(())
}
