mod close;
mod open;
mod refresh;
mod toggle;
mod view;

pub use close::Close;
pub use open::Open;
pub use refresh::Refresh;
pub use toggle::Toggle;

use std::sync::{LazyLock, RwLock};

use nvim_oxi as nvim;

use nvim::Object;
use nvim::api::opts::{OptionOpts, OptionScope};
use nvim::api::{self, Buffer};

use crate::bail;

pub static STATE: LazyLock<RwLock<State>> = LazyLock::new(|| RwLock::new(State::default()));

/// Represents internal information that can be shared between function calls.
#[derive(Debug, Clone, Default)]
pub struct State {
    /// Whether the extended help message is being displayed.
    pub display_help: bool,
}

// Technically, there is no difference between `pub` and `pub(crate)` since the consumer of this
// library is Lua, not Rust. However, semantically, I'm using `pub` to indicate the function
// is intended to be exported by the main dictionary in `lib.rs`, while `pub(crate)` indicates
// it is just an internal helper function.

/// Hide or reveal the extended help message.
#[deny(dead_code, reason = "should be added to the Dictionary in lib.rs")]
pub fn toggle_help(_: Object) {
    match STATE.write() {
        Ok(mut state) => {
            state.display_help = !state.display_help;
        }
        Err(err) => bail!("failed to acquire lock: {err}"),
    }

    let Some(mut buffer) = get_drawer_buffer() else {
        bail!("failed to get Mail UI buffer");
    };

    render(&mut buffer);
}

/// Checks if `buffer` has the properties expected of the Mail UI drawer.
pub(crate) fn is_drawer(buffer: Buffer) -> bool {
    let opts = OptionOpts::builder()
        .scope(OptionScope::Local)
        .buffer(buffer)
        .build();

    let value = api::get_option_value::<String>("filetype", &opts);
    value.is_ok_and(|filetype| &filetype == "mail-ui")
}

/// Loops through the open buffers to find the Mail UI drawer.
pub(crate) fn get_drawer_buffer() -> Option<Buffer> {
    api::list_bufs().find(|buffer| is_drawer(buffer.clone()))
}

/// Writes content into the target `buffer`.
//
// XXX(Nic): This function can abstract away the logic for unlocking the buffer to allow writing
// to it, but the actual contents should be generated using a view. We need to make a trait
// for Views so this can be turned into a generic.
pub(crate) fn render(buffer: &mut Buffer) {
    let mut replacement = vec!["Press ? for help", ""];

    let Ok(state) = STATE.read() else {
        bail!("failed to acquire lock");
    };

    if state.display_help {
        replacement.push("q - Exit");
        replacement.push("");
    }

    let opts = OptionOpts::builder().scope(OptionScope::Local).build();
    if let Err(err) = api::set_option_value("modifiable", true, &opts) {
        bail!("failed to set option value: {err}");
    }

    // An unbound range on both ends (i.e., `..`) means to replace the whole buffer.
    if let Err(err) = buffer.set_lines(.., false, replacement) {
        bail!("failed to update buffer content: {err}");
    }

    if let Err(err) = api::set_option_value("modifiable", false, &opts) {
        bail!("failed to set option value: {err}");
    }
}
