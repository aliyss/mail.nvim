use nvim_oxi as nvim;

use nvim::api::types::CommandArgs;

use crate::commands::UserCommand;

pub struct Close;

impl UserCommand for Close {
    const NAME: &'static str = "MailUIClose";
    const DESCRIPTION: &'static str = "Close the Mail UI drawer";

    fn callback(_args: CommandArgs) {
        nvim::print!("not implemented yet");
    }
}
