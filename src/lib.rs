#![warn(
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::pedantic
)]

mod commands;
mod macros;
mod state;

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
}
