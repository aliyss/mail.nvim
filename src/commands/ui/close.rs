use super::get_drawer_buffer;
use crate::commands::prelude::*;

pub struct Close;

impl UserCommand for Close {
    const NAME: Name = Name::new("MailUIClose");
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
