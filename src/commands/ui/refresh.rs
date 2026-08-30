use super::drawer::load_drawer;
use super::get_drawer_buffer;
use crate::api::config::Config;
use crate::api::file::TryFile;
use crate::commands::prelude::*;

pub struct Refresh;

impl UserCommand for Refresh {
    const NAME: Name = Name::new("MailUIRefresh");
    const DESCRIPTION: &'static str = "Refresh the contents of the Mail UI drawer";

    fn callback(_args: CommandArgs) {
        let Some(buffer) = get_drawer_buffer() else {
            nvim::print!("Mail UI is not open.");
            return;
        };

        let config = match Config::read_from_file(None) {
            Ok(config) => config,
            Err(err) => {
                nvim::print!("failed to read config file: {err}");
                return;
            }
        };

        load_drawer(buffer, config);
    }
}
