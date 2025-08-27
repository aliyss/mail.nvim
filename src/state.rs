use std::sync::{LazyLock, RwLock};

pub static STATE: LazyLock<RwLock<State>> = LazyLock::new(|| RwLock::new(State::default()));

/// Represents internal information that can be shared between function calls.
#[derive(Debug, Clone, Default)]
pub struct State {
    /// Whether the additional help message is being displayed.
    pub display_help: bool,
}
