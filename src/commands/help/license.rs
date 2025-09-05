use crate::commands::prelude::*;

pub struct License;

impl UserCommand for License {
    const NAME: Name = Name::new("MailUILicense");

    fn callback(_: CommandArgs) {
        nvim::print!("not implemented yet");
    }
}
