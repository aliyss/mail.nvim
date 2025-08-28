use nvim_oxi as nvim;

use nvim::api::types::CommandArgs;

use crate::commands::UserCommand;

pub struct Toggle;

impl UserCommand for Toggle {
    const NAME: &'static str = "MailUIToggle";
    const DESCRIPTION: &'static str = "Open/close the Mail UI drawer";

    fn callback(_args: CommandArgs) {
        nvim::print!("not implemented yet");
    }
}
