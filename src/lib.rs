pub mod api;
pub mod commands;

use nvim_oxi::{
    self as nvim,
    api::types::{WindowBorder, WindowConfig, WindowRelativeTo, WindowStyle},
};

use crate::api::help::help::Help;

#[nvim::plugin]
fn mail_nvim() -> nvim::Result<()> {
    nvim::print!("Hello from mail.nvim ...");
    let mut buf = nvim::api::create_buf(false, true)?;
    buf.set_name("Mail.nvim Help")?;
    let help_text = Help::help();
    let lines: Vec<&str> = help_text.lines().collect();
    buf.set_lines(.., false, lines)?;
    let window_config = WindowConfig::builder()
        .relative(WindowRelativeTo::Cursor)
        .row(1)
        .col(1)
        .width(100)
        .height(30)
        .focusable(true)
        .style(WindowStyle::Minimal)
        .border(WindowBorder::Rounded)
        .build();
    let _ = nvim::api::open_win(&buf, true, &window_config);
    Ok(())
}
