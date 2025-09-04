use nvim_oxi as nvim;

use nvim::api::types::CommandArgs;

use crate::commands::UserCommand;
use crate::commands::ui::{Close, Open, get_drawer_buffer};

pub struct Toggle;

impl UserCommand for Toggle {
    const NAME: &'static str = "MailUIToggle";
    const DESCRIPTION: &'static str = "Open/close the Mail UI drawer";

    fn callback(args: CommandArgs) {
        if get_drawer_buffer().is_some() {
            Close::callback(args);
        } else {
            Open::callback(args);
        }
    }
}
