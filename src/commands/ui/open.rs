use nvim_oxi as nvim;

use nvim::Object;
use nvim::api::opts::{OptionOpts, OptionScope, SetKeymapOpts};
use nvim::api::types::{CommandArgs, Mode};
use nvim::api::{self, Buffer};

use crate::bail;
use crate::commands::UserCommand;
use crate::state::STATE;

pub struct Open;

impl UserCommand for Open {
    const NAME: &'static str = "MailUI";
    const DESCRIPTION: &'static str = "Opens the Mail UI drawer";

    fn callback(_: CommandArgs) {
        if get_buffer().is_some() {
            return; // Drawer is already open.
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
            // Allows users to use `ftplugin` to style the buffer.
            ("filetype", Object::from("mail_ui")),
            // Prevents users from saving the file.
            ("buftype", Object::from("nofile")),
            // Line numbers are not relevant in this buffer.
            ("number", Object::from(false)),
            ("relativenumber", Object::from(false)),
            // Prevent users from entering INSERT mode.
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

        temporary_render_function(&mut buffer);
    }
}

pub fn toggle_help(_: Object) {
    match STATE.write() {
        Ok(mut state) => {
            state.display_help = !state.display_help;
        }
        Err(err) => bail!("failed to acquire lock: {err}"),
    }

    let Some(mut buffer) = get_buffer() else {
        bail!("failed to get Mail UI buffer");
    };

    temporary_render_function(&mut buffer);
}

fn is_drawer(buffer: Buffer) -> bool {
    let opts = OptionOpts::builder()
        .scope(OptionScope::Local)
        .buffer(buffer)
        .build();

    let value = api::get_option_value::<String>("filetype", &opts);
    value.is_ok_and(|filetype| &filetype == "mail_ui")
}

fn get_buffer() -> Option<Buffer> {
    api::list_bufs().find(|buffer| is_drawer(buffer.clone()))
}

fn temporary_render_function(buffer: &mut Buffer) {
    if !is_drawer(buffer.clone()) {
        return;
    }

    let mut replacement = vec!["Press ? for help", ""];

    let Ok(state) = STATE.read() else {
        bail!("failed to acquire lock");
    };

    if state.display_help {
        replacement.push("q - Close the Mail UI drawer");
        replacement.push("");
    }

    let opts = OptionOpts::builder().scope(OptionScope::Local).build();
    if let Err(err) = api::set_option_value("modifiable", true, &opts) {
        bail!("failed to set option value: {err}");
    }

    // An unbound range on both ends (i.e., `..`) means to replace the whole buffer.
    if let Err(err) = buffer.set_lines(.., false, replacement) {
        bail!("failed to update buffer content: {err}");
    }

    if let Err(err) = api::set_option_value("modifiable", false, &opts) {
        bail!("failed to set option value: {err}");
    }
}
