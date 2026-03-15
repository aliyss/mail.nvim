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
pub mod constants;
pub mod macros;
pub mod providers;
pub mod utils;

use nvim_oxi::{
    self as nvim,
    api::opts::{CreateAutocmdOpts, ExecOpts},
};

use nvim::{Dictionary, Function, Object};

use commands::ui::toggle_help;
use flashlog::Logger;

#[nvim::plugin]

fn mail_nvim() -> Dictionary {
    let _ = Logger::initialize()
        .with_file("logs", "message")
        .expect("failed to initialize logger")
        .launch();

    let syntax_table = include_str!("./syntax/mail-table.vim");

    let opts = CreateAutocmdOpts::builder()
        .patterns(["mail-table"])
        .callback(move |_| -> nvim::Result<bool> {
            let exec_opts = ExecOpts::builder().output(false).build();
            let _ = nvim::api::exec2(syntax_table, &exec_opts);

            Ok(false)
        })
        .build();

    nvim::api::create_autocmd(["FileType"], &opts).expect("failed to set up syntax highlighting");

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
