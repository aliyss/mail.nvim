use crate::commands::prelude::*;

pub struct Refresh;

impl UserCommand for Refresh {
    const NAME: Name = Name::new("MailUIRefresh");
    const DESCRIPTION: &'static str = "Refresh the contents of the Mail UI drawer";

    fn callback(_args: CommandArgs) {
        nvim::print!("not implemented yet");
    }
}
