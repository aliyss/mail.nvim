//! Commands toggling the "user handholding" settings, i.e. whether risky
//! actions require a confirmation popup before running.
//!
//! See the README:
//!
//! - `:MailConfigUserHandHoldingSwitchOn t/f` — risky (`!`) actions require
//!   confirmation;
//! - `:MailConfigUserHandHandHoldingSwitchOn t/f` — extra risky (`!!`) actions
//!   require confirmation.

use crate::api::config::Config;
use crate::api::file::{TryFile, prepare_default_data_directory};
use crate::commands::{completion, prelude::*};

/// Parses the `t`/`f` argument of the switch commands.
fn parse_enabled(args: &CommandArgs) -> Option<bool> {
    match args.fargs.first()?.as_str() {
        "t" | "true" | "on" | "1" => Some(true),
        "f" | "false" | "off" | "0" => Some(false),
        _ => None,
    }
}

/// Persists `config` to the default config file.
///
/// # Errors
///
/// Returns an error if the config directory cannot be located or the file
/// cannot be written.
fn save(config: &Config) -> anyhow::Result<()> {
    let directory = prepare_default_data_directory()?;
    config.write_to_file(directory.join(Config::FILE_NAME))?;
    Ok(())
}

pub struct UserHandHoldingSwitchOn;

impl UserCommand for UserHandHoldingSwitchOn {
    const NAME: Name = Name::new("MailConfigUserHandHoldingSwitchOn");
    const DESCRIPTION: &'static str = "Risky actions require confirmation";

    fn complete(arg_lead: &str, _cmd_line: &str, _cursor_pos: usize) -> Vec<String> {
        completion::filter(arg_lead, completion::booleans())
    }

    fn callback(args: CommandArgs) {
        let Some(enabled) = parse_enabled(&args) else {
            bail!("usage: MailConfigUserHandHoldingSwitchOn <t|f>");
        };

        let mut config = match Config::read_from_file(None) {
            Ok(config) => config,
            Err(err) => bail!("failed to read config: {err:#}"),
        };

        config.set_user_handholding(enabled);

        match save(&config) {
            Ok(()) => nvim_oxi::print!(
                "User hand holding is now {}",
                if enabled { "enabled" } else { "disabled" }
            ),
            Err(err) => nvim_oxi::print!("failed to save config: {err:#}"),
        }
    }
}

pub struct UserHandHandHoldingSwitchOn;

impl UserCommand for UserHandHandHoldingSwitchOn {
    const NAME: Name = Name::new("MailConfigUserHandHandHoldingSwitchOn");
    const DESCRIPTION: &'static str = "Extra risky actions require confirmation";

    fn complete(arg_lead: &str, _cmd_line: &str, _cursor_pos: usize) -> Vec<String> {
        completion::filter(arg_lead, completion::booleans())
    }

    fn callback(args: CommandArgs) {
        let Some(enabled) = parse_enabled(&args) else {
            bail!("usage: MailConfigUserHandHandHoldingSwitchOn <t|f>");
        };

        let mut config = match Config::read_from_file(None) {
            Ok(config) => config,
            Err(err) => bail!("failed to read config: {err:#}"),
        };

        config.set_user_handhandholding(enabled);

        match save(&config) {
            Ok(()) => nvim_oxi::print!(
                "User hand hand holding is now {}",
                if enabled { "enabled" } else { "disabled" }
            ),
            Err(err) => nvim_oxi::print!("failed to save config: {err:#}"),
        }
    }
}
