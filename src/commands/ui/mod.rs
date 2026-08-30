mod close;
pub mod drawer;
mod open;
mod refresh;
mod toggle;
pub mod view;

pub use close::Close;
pub use open::Open;
pub use refresh::Refresh;
pub use toggle::Toggle;

use std::collections::HashMap;

use nvim_oxi::Object;
use nvim_oxi::api::opts::{OptionOpts, OptionScope, SetKeymapOpts};
use nvim_oxi::api::types::Mode;
use nvim_oxi::api::{self, Buffer};

use crate::api::config::ui::view::{UiViewComponent, UiViewComponentContext, UiViewComponentType};
use crate::utils::buffer::metadata::BufferMetadata;
use crate::utils::buffer::render::FromBuffer;

/// The component rendered into the Mail UI drawer: the account list.
#[must_use]
pub(crate) fn drawer_component() -> UiViewComponent {
    UiViewComponent {
        id: "mail-ui-drawer".into(),
        name: "Accounts".into(),
        component_type: UiViewComponentType::Drawer,
        context: UiViewComponentContext {
            command_group: "Account".into(),
            command_type: "List".into(),
            arguments: HashMap::new(),
            context: Vec::new(),
        },
        layout: None,
        on_enter: None,
        link: None,
    }
}

/// Checks if `buffer` has the properties expected of the Mail UI drawer.
pub(crate) fn is_drawer(buffer: Buffer) -> bool {
    let opts = OptionOpts::builder()
        .scope(OptionScope::Local)
        .buf(buffer)
        .build();

    let value = api::get_option_value::<String>("filetype", &opts);
    // `mail-ui` is set when the drawer is created, `mail-drawer` once the
    // drawer component has been rendered into it.
    value.is_ok_and(|filetype| matches!(filetype.as_str(), "mail-ui" | "mail-drawer"))
}

/// Loops through the open buffers to find the Mail UI drawer.
pub(crate) fn get_drawer_buffer() -> Option<Buffer> {
    api::list_bufs().find(|buffer| is_drawer(buffer.clone()))
}

/// Checks whether any mail UI buffer (the drawer or a rendered view
/// component) is currently open.
pub(crate) fn has_open_mail_buffer() -> bool {
    api::list_bufs().any(|buffer| {
        is_drawer(buffer.clone()) || BufferMetadata::from_buffer(&buffer, None).is_ok()
    })
}

/// Configures `buffer` as a Mail UI drawer pane: options, filetype and the
/// drawer navigation keymaps.
///
/// # Errors
///
/// Returns an error if an option or keymap cannot be set.
pub(crate) fn setup_drawer_buffer(buffer: &mut Buffer) -> anyhow::Result<()> {
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
            anyhow::bail!("failed to set option value: {err}");
        }
    }

    let keymaps: [(Mode, &'static str, &'static str); 9] = [
        // Toggle the node under the cursor (account/folder) or open the
        // action's content.
        (
            Mode::Normal,
            "<CR>",
            ":lua require('mail_nvim').drawer_action()<CR>",
        ),
        (
            Mode::Normal,
            "o",
            ":lua require('mail_nvim').drawer_action()<CR>",
        ),
        // Refresh the drawer.
        (Mode::Normal, "R", ":MailUIRefresh<CR>"),
        // Close the Mail UI drawer.
        (Mode::Normal, "q", ":bdelete<CR>"),
        // Show context-sensitive help.
        (
            Mode::Normal,
            "?",
            ":lua require('mail_nvim').show_help()<CR>",
        ),
        // Navigate between siblings.
        (
            Mode::Normal,
            "J",
            ":lua require('mail_nvim').drawer_goto_sibling(1)<CR>",
        ),
        (
            Mode::Normal,
            "K",
            ":lua require('mail_nvim').drawer_goto_sibling(-1)<CR>",
        ),
        // Navigate to the first child / the parent node.
        (
            Mode::Normal,
            "<C-n>",
            ":lua require('mail_nvim').drawer_goto_node(1)<CR>",
        ),
        (
            Mode::Normal,
            "<C-p>",
            ":lua require('mail_nvim').drawer_goto_node(-1)<CR>",
        ),
    ];

    let opts = SetKeymapOpts::builder().silent(true).build();

    for (mode, keys, command) in keymaps {
        if let Err(err) = buffer.set_keymap(mode, keys, command, &opts) {
            anyhow::bail!("failed to set keymap: {err}");
        }
    }

    Ok(())
}
