use std::collections::HashMap;

use nvim_oxi::Object;

use crate::{
    api::{
        config::{
            Config,
            ui::view::{UiViewComponent, UiViewComponentContext, UiViewComponentType},
        },
        file::TryFile,
    },
    commands::prelude::*,
    utils::render::{ASYNC_RUNTIME, render_ui_view_from_component},
};

pub struct FolderList;

impl UserCommand for FolderList {
    const NAME: Name = Name::new("MailFolderList");
    const DESCRIPTION: &'static str = "List all folders in a mail account";

    fn callback(_: CommandArgs) {
        let current_buffer = api::get_current_buf();

        let in_buffer_list = false;
        let is_temporary = true;
        let mut buffer = match api::create_buf(in_buffer_list, is_temporary) {
            Ok(buffer) => buffer,
            Err(err) => bail!("failed to create buffer: {err}"),
        };

        if let Err(err) = api::set_current_buf(&buffer) {
            bail!("failed to set current buffer: {err}");
        }

        let options: [(&'static str, Object); 3] = [
            // Allows users to use `ftplugin` to customize the buffer.
            ("filetype", Object::from("mail-table")),
            // Prevents users from saving the file.
            ("buftype", Object::from("nofile")),
            // Prevents users from entering INSERT mode.
            ("modifiable", Object::from(false)),
        ];

        let opts = OptionOpts::builder().scope(OptionScope::Local).build();

        for (name, value) in options {
            if let Err(err) = api::set_option_value(name, value, &opts) {
                bail!("failed to set option value: {err}");
            }
        }

        let keymaps: [(Mode, &'static str, &'static str); 1] =
            [(Mode::Normal, "q", ":bdelete<CR>")];

        let opts = SetKeymapOpts::builder().silent(true).build();

        for (mode, keys, command) in keymaps {
            if let Err(err) = buffer.set_keymap(mode, keys, command, &opts) {
                bail!("failed to set keymap: {err}");
            }
        }

        let config = Config::read_from_file(None)
            .expect("failed to read config file")
            .clone();

        ASYNC_RUNTIME.block_on(async move {
            render_ui_view_from_component(
                buffer,
                Some(current_buffer),
                UiViewComponent {
                    id: "command-folder-list".into(),
                    name: "FolderList".into(),
                    component_type: UiViewComponentType::Table,
                    context: UiViewComponentContext {
                        command_group: "Folder".into(),
                        command_type: "List".into(),
                        arguments: HashMap::new(),
                        context: HashMap::new(),
                    },
                    layout: None,
                },
                config,
            )
            .await;
        });
    }
}
