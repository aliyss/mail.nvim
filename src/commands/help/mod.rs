mod about;
mod changelog;
pub mod contribute;
pub mod feature_request;
pub mod issue_report;
pub mod keybindings;
pub mod license;
pub mod support;

pub use about::About;
pub use changelog::Changelog;

use nvim::api::types::{WindowBorder, WindowConfig, WindowRelativeTo, WindowStyle};

use crate::api::help::HelpMessage;
use crate::commands::prelude::*;

pub struct Help;

impl UserCommand for Help {
    const NAME: Name = Name::new("MailUIHelp");
    const DESCRIPTION: &'static str = "List all available commands/options";

    fn callback(_: CommandArgs) {
        let in_buffer_list = false;
        let is_temporary = true;
        let mut buffer = match api::create_buf(in_buffer_list, is_temporary) {
            Ok(buffer) => buffer,
            Err(err) => bail!("failed to create buffer: {err}"),
        };

        let help = HelpMessage::text();

        if let Err(err) = buffer.set_lines(.., false, help.lines()) {
            bail!("failed to update buffer content: {err}");
        }

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

        if let Err(err) = api::open_win(&buffer, true, &window_config) {
            bail!("failed to open window: {err}");
        }

        let keymaps = [
            // Close the window.
            (Mode::Normal, "q", ":bdelete<CR>"),
        ];

        let opts = SetKeymapOpts::builder().silent(true).build();

        for (mode, keys, command) in keymaps {
            if let Err(err) = buffer.set_keymap(mode, keys, command, &opts) {
                bail!("failed to set keymap: {err}");
            }
        }
    }
}
