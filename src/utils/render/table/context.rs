use nvim_oxi::api::{self, Buffer};

use crate::utils::{
    buffer::render::FromBuffer,
    render::table::render::{RenderTable, Table},
};

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
        Err(_e) => {
            anyhow::bail!("failed to parse table data from buffer");
        }
    };

    let result = table_data.data.get(row - table_data.offset - 1).cloned();

    match result {
        Some(item) => Ok(item),
        None => anyhow::bail!("failed to fetch row data from buffer: no data at row {row}"),
    }
}
