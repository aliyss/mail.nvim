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
pub mod tests;
pub mod utils;

use nvim_oxi::{self as nvim, Dictionary, Function, Object};

use commands::email::pagination::email_list_page;
use commands::email::selection::{
    email_clear_selection, email_select_visual_range, email_toggle_selection,
};
use commands::ui::drawer::{drawer_action, drawer_goto_node, drawer_goto_sibling};
use commands::ui::view::engine::recalculate_layout;
use commands::ui::view::navigation::{pane_selection_changed, ui_enter};
use flashlog::Logger;
use utils::confirm::{confirm_no, confirm_yes};
use utils::popup::show_help;

#[nvim::plugin]

fn mail_nvim() -> Dictionary {
    let _ = Logger::initialize()
        .with_file("logs", "message")
        .expect("failed to initialize logger")
        .launch();

    let dictionary = Dictionary::from_iter([
        (
            "drawer_action",
            Object::from(lua_function("drawer_action", drawer_action)),
        ),
        (
            "drawer_goto_sibling",
            Object::from(lua_function("drawer_goto_sibling", drawer_goto_sibling)),
        ),
        (
            "drawer_goto_node",
            Object::from(lua_function("drawer_goto_node", drawer_goto_node)),
        ),
        (
            "show_help",
            Object::from(lua_function("show_help", show_help)),
        ),
        (
            "email_list_page",
            Object::from(lua_function("email_list_page", email_list_page)),
        ),
        (
            "email_toggle_selection",
            Object::from(lua_function(
                "email_toggle_selection",
                email_toggle_selection,
            )),
        ),
        (
            "email_clear_selection",
            Object::from(lua_function("email_clear_selection", email_clear_selection)),
        ),
        (
            "email_select_visual_range",
            Object::from(lua_function(
                "email_select_visual_range",
                email_select_visual_range,
            )),
        ),
        ("ui_enter", Object::from(lua_function("ui_enter", ui_enter))),
        (
            "recalculate_layout",
            Object::from(lua_function("recalculate_layout", recalculate_layout)),
        ),
        (
            "pane_selection_changed",
            Object::from(lua_function(
                "pane_selection_changed",
                pane_selection_changed,
            )),
        ),
        (
            "confirm_yes",
            Object::from(lua_function("confirm_yes", confirm_yes)),
        ),
        (
            "confirm_no",
            Object::from(lua_function("confirm_no", confirm_no)),
        ),
    ]);

    if let Err(err) = commands::register_commands() {
        nvim::print!("failed to register commands: {err}");
        return dictionary;
    }

    // Keep the pane sizes of an open view applied to their component layouts
    // whenever a window changes (a mail is opened or closed, a split is
    // resized): recompute the widths and re-render the affected panes.
    //
    // Registered through the typed `nvim_create_autocmd` API instead of a
    // raw `:autocmd` line: in an `:autocmd` definition a `|` is stored
    // literally in the command, so a trailing `|augroup END` would end up
    // inside the `:lua` chunk and break it with E5107.
    if let Err(err) = nvim_oxi::api::create_augroup("MailNvimLayout", &Default::default())
        .and_then(|_| {
            nvim_oxi::api::create_autocmd(
                ["WinNew", "WinClosed", "WinEnter", "WinResized"],
                &nvim_oxi::api::opts::CreateAutocmdOpts::builder()
                    .group("MailNvimLayout")
                    .patterns(["*"])
                    .command("lua require('mail_nvim').recalculate_layout()")
                    .build(),
            )
        })
    {
        nvim::print!("failed to register layout autocmds: {err}");
    }

    dictionary
}

/// Wraps a Lua-exported Rust function so that a panic cannot unwind across
/// the Lua boundary and crash (or corrupt) Neovim.
fn lua_function<F>(name: &'static str, f: F) -> Function<Object, ()>
where
    F: Fn(Object) + Copy + 'static,
{
    Function::from_fn(move |arg: Object| {
        if let Err(payload) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(arg))) {
            nvim::print!("lua function `{name}` panicked: {}", panic_message(payload));
        }
    })
}

/// Formats a panic payload into a message that can be shown to the user.
#[allow(clippy::needless_pass_by_value)] // `payload` is inspected, never moved.
pub(crate) fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }

    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }

    "unknown panic".into()
}
