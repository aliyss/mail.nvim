pub mod config;
pub mod help;
pub mod ui;

use nvim_oxi as nvim;

use nvim::api;
use nvim::api::opts::CreateCommandOpts;
use nvim::api::types::CommandArgs;

use ui::Open;

pub trait UserCommand
where
    Self: 'static,
{
    /// The name of the command to be executed (e.g., `"MailUI"` for `:MailUI`).
    const NAME: &'static str;

    /// A brief explaination of the command.
    const DESCRIPTION: &'static str;

    /// Create a new user command and register it to Neovim.
    ///
    /// The default implementation registers a new command with only a name and description.
    fn register() -> Result<(), api::Error> {
        let opts = CreateCommandOpts::builder().desc(Self::DESCRIPTION).build();
        api::create_user_command(Self::NAME, Self::callback, &opts)
    }

    /// The implementation of the command.
    fn callback(args: CommandArgs);
}

pub fn register_commands() -> Result<(), api::Error> {
    Open::register()?;
    Ok(())
}
