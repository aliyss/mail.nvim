use nvim_oxi::api::Buffer;

use crate::utils::{
    buffer::render::FromBuffer,
    render::message::render::{Message, RenderMessage},
};

pub fn fetch_data_from_buffer<T>(buf: &Buffer, metadata_offset: usize) -> anyhow::Result<T>
where
    T: RenderMessage,
    T::Item: Clone,
{
    let table_data = match Message::<T>::from_buffer(buf, Some(metadata_offset)) {
        Ok(data) => data,
        Err(_e) => {
            anyhow::bail!("failed to parse table data from buffer");
        }
    };

    Ok(table_data.data)
}
