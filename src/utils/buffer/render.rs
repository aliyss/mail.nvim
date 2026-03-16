use nvim_oxi::api::Buffer;

pub trait FromBuffer {
    fn from_buffer(buffer: &Buffer, line_offset: Option<usize>) -> anyhow::Result<Self>
    where
        Self: Sized;
}

pub trait ToBuffer {
    fn to_buffer(self, buffer: &mut Buffer, line_offset: usize) -> anyhow::Result<Self>
    where
        Self: Sized;
}
