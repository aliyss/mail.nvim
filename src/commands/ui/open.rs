use nvim_oxi as nvim;

use nvim::Object;
use nvim::api;
use nvim::api::opts::{OptionOpts, OptionScope, SetKeymapOpts};
use nvim::api::types::{CommandArgs, Mode};

use crate::bail;
use crate::commands::UserCommand;
use crate::commands::ui::{get_buffer, render};

pub struct Open;

impl UserCommand for Open {
    const NAME: &'static str = "MailUI";
    const DESCRIPTION: &'static str = "Opens the Mail UI drawer";

    fn callback(_: CommandArgs) {
        if get_buffer().is_some() {
            return; // The drawer is already open.
        }

        let in_buffer_list = false;
        let is_temporary = true;
        let mut buffer = match api::create_buf(in_buffer_list, is_temporary) {
            Ok(buffer) => buffer,
            Err(err) => bail!("failed to create buffer: {err}"),
        };

        if let Err(err) = api::command("vsplit") {
            bail!("failed to create a vertical split: {err}");
        }

        if let Err(err) = api::command("vertical topleft resize 40") {
            bail!("failed to resize window: {err}");
        }

        if let Err(err) = api::set_current_buf(&buffer) {
            bail!("failed to set current buffer: {err}");
        }

        let options: [(&'static str, Object); 5] = [
            // Allows users to use `ftplugin` to customize the buffer.
            ("filetype", Object::from("mail-ui")),
            // Prevents users from saving the file.
            ("buftype", Object::from("nofile")),
            // Line numbers are not relevant in this buffer.
            ("number", Object::from(false)),
            ("relativenumber", Object::from(false)),
            // Prevents users from entering INSERT mode.
            ("modifiable", Object::from(false)),
        ];

        let opts = OptionOpts::builder().scope(OptionScope::Local).build();

        for (name, value) in options {
            if let Err(err) = api::set_option_value(name, value, &opts) {
                bail!("failed to set option value: {err}");
            }
        }

        let keymaps: [(Mode, &'static str, &'static str); 2] = [
            // Close the Mail UI drawer.
            (Mode::Normal, "q", ":bd<CR>"),
            // Toggle the extended help message.
            (
                Mode::Normal,
                "?",
                ":lua require('mail_nvim').toggle_help()<CR>",
            ),
        ];

        let opts = SetKeymapOpts::builder().silent(true).build();

        for (mode, keys, command) in keymaps {
            if let Err(err) = buffer.set_keymap(mode, keys, command, &opts) {
                bail!("failed to set keymap: {err}");
            }
        }

        render(&mut buffer);
    }
}
