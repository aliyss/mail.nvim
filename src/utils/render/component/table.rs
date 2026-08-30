//! Renders list data as a table (accounts, folders, emails, messages).
//!
//! Every list table gets a leading `Sel` column: emails and threads use it
//! for the multi-select marker, and while a row is loading (its content is
//! being fetched after an enter action) the cell shows the animated spinner
//! instead. Accounts and folders get the column too, so the spinner has a
//! home even though they do not support multi-select yet.

use nvim_oxi::api::Buffer;
use nvim_oxi::api::types::Mode;

use crate::api::config::ui::view::UiViewComponent;
use crate::api::email::EmailMessage;
use crate::utils::buffer::metadata::BufferMetadata;
use crate::utils::buffer::render::ToBuffer;
use crate::utils::loading::{self, Anchor};
use crate::utils::render::ComponentData;
use crate::utils::render::component::{
    Keymap, email_action_keymaps, email_selection_keymaps, email_visual_action_keymaps,
    list_enter_keymaps,
};
use crate::utils::render::table::marked::{HasId, MarkedTable};
use crate::utils::render::table::render::{RenderTable, Table};
use crate::utils::selection;

/// # Errors
///
/// Returns an error if the table fails to render into `buffer`.
pub fn render(
    _component: &UiViewComponent,
    data: ComponentData,
    buffer: &mut Buffer,
    metadata: &BufferMetadata,
) -> anyhow::Result<Vec<Keymap>> {
    // Rows of this table that are loading (an enter action opened them): draw
    // their spinner in the `Sel` column.
    let spinner_rows: Vec<(usize, &'static str)> = loading::spinners(buffer)
        .into_iter()
        .filter_map(|(anchor, frame)| match anchor {
            Anchor::Row(row) => Some((row, frame)),
            Anchor::Account(_) | Anchor::Action { .. } => None,
        })
        .collect();

    let mut keymaps = Vec::new();

    match data {
        ComponentData::Accounts(accounts) => {
            let table = render_list_table(accounts, buffer, metadata, &spinner_rows)?;

            keymaps.extend(list_enter_keymaps(
                table.offset,
                table.data.len(),
                metadata.line_count,
                "No account selected",
            ));
        }
        ComponentData::Folders(folders) => {
            let table = render_list_table(folders, buffer, metadata, &spinner_rows)?;

            keymaps.extend(list_enter_keymaps(
                table.offset,
                table.data.len(),
                metadata.line_count,
                "No folder selected",
            ));
        }
        ComponentData::Emails(emails) => {
            keymaps.extend(render_email_list(emails, buffer, metadata, &spinner_rows)?);
        }
        ComponentData::Threads(emails) => {
            keymaps.extend(render_email_list(emails, buffer, metadata, &spinner_rows)?);
        }
        ComponentData::EmailMessages(email_messages) => {
            let table = Table::<Vec<EmailMessage>>::new(email_messages);
            table.to_buffer(buffer, metadata.line_count)?;
        }
        ComponentData::None => {
            nvim_oxi::print!("None rendering not implemented yet.");
        }
    }

    Ok(keymaps)
}

/// Renders an email list/thread table with the selection marker column and
/// returns the enter, multi-select, action and pagination keymaps for it.
fn render_email_list<T>(
    data: T,
    buffer: &mut Buffer,
    metadata: &BufferMetadata,
    spinner_rows: &[(usize, &'static str)],
) -> anyhow::Result<Vec<Keymap>>
where
    T: RenderTable,
    T::Item: HasId,
{
    let table = render_list_table(data, buffer, metadata, spinner_rows)?;

    let mut keymaps = Vec::new();
    keymaps.extend(list_enter_keymaps(
        table.offset,
        table.data.len(),
        metadata.line_count,
        "No email selected",
    ));
    keymaps.extend(email_selection_keymaps(
        table.offset,
        table.data.len(),
        metadata.line_count,
        "No email selected",
    ));
    keymaps.extend(email_action_keymaps());
    keymaps.extend(email_visual_action_keymaps());
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

    Ok(keymaps)
}

/// Renders a list table with the `Sel` column, drawing the spinner in the
/// `Sel` cell of every loading row, and records where each spinner was drawn
/// so the animation can update (and eventually restore) the cell in place.
fn render_list_table<T>(
    data: T,
    buffer: &mut Buffer,
    metadata: &BufferMetadata,
    spinner_rows: &[(usize, &'static str)],
) -> anyhow::Result<Table<MarkedTable<T>>>
where
    T: RenderTable,
    T::Item: HasId,
{
    let selected = selection::selected_ids(buffer);
    let mut marked = MarkedTable::new(data, selected.clone());
    for (row, frame) in spinner_rows {
        marked = marked.with_spinner(*row, frame.chars().next().unwrap_or('⠼'));
    }

    let table = Table::<MarkedTable<T>>::new(marked)
        .to_buffer(buffer, metadata.line_count)
        .map_err(|err| anyhow::anyhow!("failed to render table: {err}"))?;

    for (row, frame) in spinner_rows {
        let row = *row;
        // The data row `row` sits at `metadata.line_count + table.offset + row`
        // (0-based): `table.offset` covers the header and separator lines, and
        // the metadata block precedes them.
        let line = metadata.line_count + table.offset + row;
        if line >= buffer.line_count().unwrap_or(0) {
            continue;
        }
        let line_text = buffer
            .get_lines(line..=line, true)
            .ok()
            .and_then(|mut lines| lines.next())
            .map(|line| line.to_string())
            .unwrap_or_default();
        let Some(column) = line_text.find(*frame) else {
            continue;
        };
        // What was under the spinner before it was drawn: the selected marker
        // or a blank cell. Must be a single column wide, like the spinner
        // itself, or the line shrinks when the cell is restored.
        let replaced = table.data.get(row).map_or_else(
            || " ".to_string(),
            |item| {
                if selected.contains(item.id()) {
                    ">".to_string()
                } else {
                    " ".to_string()
                }
            },
        );
        loading::set_position(buffer, &Anchor::Row(row), line, column, replaced);
    }

    Ok(table)
}
