use nvim_oxi as nvim;

use nvim::api;
use nvim::api::types::CommandArgs;

use crate::bail;
use crate::commands::UserCommand;
use crate::commands::ui::get_drawer_buffer;

pub struct Close;

impl UserCommand for Close {
    const NAME: &'static str = "MailUIClose";
    const DESCRIPTION: &'static str = "Close the Mail UI drawer";

    fn callback(_args: CommandArgs) {
        let Some(buffer) = get_drawer_buffer() else {
            return;
        };

        if let Err(err) = api::command(&format!("bdelete {}", buffer.handle())) {
            bail!("failed to close buffer: {err}");
        }
    }
}
