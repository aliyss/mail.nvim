//! This module exports types that are commonly used when implementing User Commands.

// The minimum types needed to implement the main [`crate::commands::UserCommand`] trait.

pub use nvim_oxi as nvim;

pub use nvim::api;
pub use nvim::api::types::CommandArgs;

pub use crate::commands::{Name, UserCommand};

// Additional convenience types. (Feel free to add to this.)

pub use nvim::api::opts::{OptionOpts, OptionScope, SetKeymapOpts};
pub use nvim::api::types::Mode;

pub use crate::bail;
