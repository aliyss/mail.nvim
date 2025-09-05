use crate::commands::prelude::*;

pub struct Changelog;

impl UserCommand for Changelog {
    const NAME: Name = Name::new("MailUIChangelog");

    fn callback(_: CommandArgs) {
        nvim::print!("not implemented yet");
    }
}
