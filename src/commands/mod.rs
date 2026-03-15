pub mod account;
pub mod config;
pub mod folder;
pub mod help;
pub mod prelude;
pub mod ui;

use nvim_oxi as nvim;

use nvim::api;
use nvim::api::opts::CreateCommandOpts;
use nvim::api::types::CommandArgs;

use account::list::AccountList;
use help::{About, Changelog, Help};
use ui::{Close, Open, Refresh, Toggle};

use crate::commands::folder::list::FolderList;

/// A trait for implementing User Commands in Neovim.
///
/// # Examples
///
/// ```
/// use mail_nvim::commands::prelude::*;
///
/// pub struct MyCommand;
///
/// impl UserCommand for MyCommand {
///     const NAME: Name = Name::new("MyCommand");
///     const DESCRIPTION: &'static str = "";
///
///     fn callback(_: CommandArgs) {
///         nvim::print!("Hello, MyCommand!");
///     }
/// }
/// ```
pub trait UserCommand
where
    Self: 'static,
{
    /// The name of the command to be executed (e.g., `"MailUI"` for `:MailUI`).
    const NAME: Name;

    /// A brief explanation of the command.
    const DESCRIPTION: &'static str = "";

    /// Create a new user command and register it to Neovim.
    ///
    /// The default implementation registers a new command with a name and description.
    ///
    /// # Errors
    ///
    /// This function may return an error if:
    ///
    /// - [`Self::NAME`] does not start with an uppercase letter.
    /// - [`Self::NAME`] contains non-alphanumeric characters.
    ///
    /// If a different error occurred, reference [`nvim_create_user_command`].
    ///
    /// [`nvim_create_user_command`]: https://github.com/neovim/neovim/blob/v0.10.0/src/nvim/api/command.c#L954
    fn register() -> Result<(), api::Error> {
        let opts = CreateCommandOpts::builder().desc(Self::DESCRIPTION).build();
        api::create_user_command(Self::NAME.0, Self::callback, &opts)
    }

    /// The implementation of the command.
    fn callback(args: CommandArgs);
}

/// Adds compile-time checks to the user command name to ensure it is valid.
pub struct Name(&'static str);

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl Name {
    /// Define a new user command name.
    ///
    /// # Panics
    ///
    /// This function panics if:
    ///
    /// - `name` is empty.
    /// - `name` does not start with an uppercase letter.
    /// - `name` contains non-alphanumeric characters.
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        assert!(!name.is_empty(), "command name cannot be empty");

        let bytes = name.as_bytes();
        assert!(
            bytes[0].is_ascii_uppercase(),
            "command name must start with an uppercase character"
        );

        let mut i = 0;
        while i < bytes.len() {
            let c = bytes[i];
            assert!(
                c.is_ascii_alphanumeric(),
                "command names may only contain alphanumeric characters"
            );
            i += 1;
        }

        Self(name)
    }
}

pub(crate) fn register_commands() -> Result<(), api::Error> {
    AccountList::register()?;
    FolderList::register()?;

    About::register()?;
    Changelog::register()?;
    Help::register()?;

    Close::register()?;
    Open::register()?;
    Refresh::register()?;
    Toggle::register()?;

    Ok(())
}
