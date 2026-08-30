//! Renders email messages as a file view (header info + body).

use nvim_oxi::api::Buffer;

use crate::api::config::ui::view::UiViewComponent;
use crate::api::email::EmailMessage;
use crate::utils::buffer::metadata::BufferMetadata;
use crate::utils::buffer::render::ToBuffer;
use crate::utils::render::ComponentData;
use crate::utils::render::component::{Keymap, email_action_keymaps};
use crate::utils::render::message::render::Message;

/// # Errors
///
/// Returns an error if the message fails to render into `buffer`.
pub fn render(
    _component: &UiViewComponent,
    data: ComponentData,
    buffer: &mut Buffer,
    metadata: &BufferMetadata,
) -> anyhow::Result<Vec<Keymap>> {
    match data {
        ComponentData::EmailMessages(email_messages) => {
            let email_message = if let Some(message) = email_messages.first() {
                message.clone()
            } else {
                nvim_oxi::print!("No email message to display.");
                return Ok(Vec::new());
            };

            let message = Message::<EmailMessage>::new(email_message);
            match message.to_buffer(buffer, metadata.line_count) {
                Ok(_) => (),
                Err(err) => {
                    nvim_oxi::print!("Failed to render email message: {err}");
                }
            }
        }
        _ => {
            nvim_oxi::print!("Message rendering for this component not implemented yet.");
        }
    }

    Ok(email_action_keymaps())
}
