use super::{Close, Open, get_drawer_buffer};
use crate::commands::prelude::*;

pub struct Toggle;

impl UserCommand for Toggle {
    const NAME: Name = Name::new("MailUIToggle");
    const DESCRIPTION: &'static str = "Open/close the Mail UI drawer";

    fn callback(args: CommandArgs) {
        if get_drawer_buffer().is_some() {
            Close::callback(args);
        } else {
            Open::callback(args);
        }
    }
}
