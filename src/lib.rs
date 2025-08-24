use nvim_oxi as nvim;

#[nvim::plugin]
fn mail_nvim() -> nvim::Result<()> {
    nvim::print!("Hello from mail.nvim ...");
    Ok(())
}
