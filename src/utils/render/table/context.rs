use nvim_oxi::api::{self, Buffer};

use crate::api::email::Email;
use crate::utils::{
    buffer::render::FromBuffer,
    render::{cached_row_email, cached_row_id},
    render::table::marked::HasId,
    render::table::render::{RenderTable, Table},
};

/// The id of the item under the cursor, preferring the pane's cached
/// structured data (the same items that rendered the table, in the same
/// order) over re-parsing the rendered text — a narrow pane truncating a
/// header can no longer break the lookup.
///
/// The fallback parses the rendered table like `fetch_row_from_buffer` did;
/// the pane's cache is always populated for table buffers (every render goes
/// through [`render_into_buffer`](crate::utils::render::render_into_buffer)),
/// but keeping the parse covers buffers that predate the cache.
///
/// # Errors
///
/// Returns an error when the cursor is not on a table row and the pane has
/// no cached data to fall back on.
pub fn fetch_row_id<T>(buf: &Buffer, metadata_line_count: usize) -> anyhow::Result<String>
where
    T: RenderTable,
    T::Item: HasId + Clone,
{
    if let Some(id) = cached_row_id(buf, metadata_line_count) {
        return Ok(id);
    }
    fetch_row_from_buffer::<T>(buf, metadata_line_count).map(|item| item.id().to_string())
}

/// The email under the cursor, preferring the pane's cached data (email list
/// or thread) over re-parsing the rendered table.
///
/// # Errors
///
/// Returns an error when the cursor is not on a table row and the pane has
/// no cached data to fall back on.
pub fn fetch_row_email(buf: &Buffer, metadata_line_count: usize) -> anyhow::Result<Email> {
    if let Some(email) = cached_row_email(buf, metadata_line_count) {
        return Ok(email);
    }
    fetch_row_from_buffer::<Vec<Email>>(buf, metadata_line_count)
}

/// Parses the rendered table in `buf` to recover the item under the cursor.
///
/// Kept as the fallback of [`fetch_row_id`]/[`fetch_row_email`]; prefer those
/// over calling this directly, since parsing depends on the table's headers
/// staying intact.
///
/// # Errors
///
/// Returns an error when the buffer cannot be parsed as a table or the
/// cursor is not within the table rows.
pub fn fetch_row_from_buffer<T>(buf: &Buffer, metadata_offset: usize) -> anyhow::Result<T::Item>
where
    T: RenderTable,
    T::Item: Clone,
{
    let (row, _) = match api::get_current_win().get_cursor() {
        Ok(pos) => pos,
        Err(_e) => {
            anyhow::bail!("failed to get cursor position from buffer");
        }
    };

    let table_data = match Table::<T>::from_buffer(buf, Some(metadata_offset)) {
        Ok(data) => data,
        Err(_e) => anyhow::bail!("failed to parse table data from buffer"),
    };

    let Some(row_index) = row.checked_sub(table_data.offset + 1) else {
        anyhow::bail!(
            "failed to fetch row data from buffer: cursor is not within the table rows"
        );
    };

    let result = table_data.data.get(row_index).cloned();

    match result {
        Some(item) => Ok(item),
        None => anyhow::bail!("failed to fetch row data from buffer: no data at row {row}"),
    }
}
