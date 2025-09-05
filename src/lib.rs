#![warn(
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::pedantic
)]

pub mod api;
pub mod commands;
pub mod macros;

use nvim_oxi as nvim;

use nvim::{Dictionary, Function, Object};

use commands::ui::toggle_help;

#[nvim::plugin]

fn mail_nvim() -> Dictionary {
    nvim::print!("Hello, mail.nvim!");
    // Unfortunately, it is impossible to avoid panics here. If the program panicked, make sure
    // the names and function signatures are correct.
    let dictionary =
        Dictionary::from_iter([("toggle_help", Object::from(Function::from_fn(toggle_help)))]);

    if let Err(err) = commands::register_commands() {
        nvim::print!("failed to register commands: {err}");
        return dictionary;
    }

    dictionary

    // NOTE: I moved this to `commands/help/mod.rs`.

    // fn mail_nvim() -> nvim::Result<()> {
    //    nvim::print!("Hello from mail.nvim ...");
    //    let mut buf = nvim::api::create_buf(false, true)?;
    //    buf.set_name("Mail.nvim Help")?;
    //    let help_text = Help::help();
    //    let lines: Vec<&str> = help_text.lines().collect();
    //    buf.set_lines(.., false, lines)?;
    //    let window_config = WindowConfig::builder()
    //        .relative(WindowRelativeTo::Cursor)
    //        .row(1)
    //        .col(1)
    //        .width(100)
    //        .height(30)
    //        .focusable(true)
    //        .style(WindowStyle::Minimal)
    //        .border(WindowBorder::Rounded)
    //        .build();
    //    let _ = nvim::api::open_win(&buf, true, &window_config);
    //    Ok(())
}
