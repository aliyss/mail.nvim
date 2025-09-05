use crate::commands::prelude::*;

pub struct About;

impl UserCommand for About {
    const NAME: Name = Name::new("MailUIAbout");
    const DESCRIPTION: &'static str = "";

    fn callback(_: CommandArgs) {
        nvim::print!("not implemented yet");
    }
}
