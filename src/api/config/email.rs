use std::collections::HashMap;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Format {
    #[default]
    Plain,
    Json,
    Html,
}

/// `ViewAs` configuration options.
#[derive(Debug, Builder, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[builder(setter(strip_option))]
pub struct ViewAs {
    /// Format of the output.
    #[builder(setter(into))]
    format: Format,

    /// Command to execute.
    #[builder(setter(into), default)]
    command: Option<String>,

    /// Should capture output.
    #[builder(setter(into), default)]
    capture_output: Option<bool>,
}

impl ViewAs {
    /// Create a builder for the endpoint.
    #[must_use]
    pub fn builder() -> ViewAsBuilder {
        ViewAsBuilder::default()
    }
}

/// `Email` configuration options.
#[derive(Debug, Clone, PartialEq, Eq, Builder, Serialize, Deserialize)]
#[builder(setter(strip_option))]
pub struct Email {
    /// `ViewAs` command configuration.
    #[builder(setter(into), default)]
    view_as_commands: Option<HashMap<String, ViewAs>>,

    /// Default `view_as` command.
    #[builder(setter(into), default)]
    view_as_commands_default: Option<String>,
}

impl Email {
    /// Create a builder for the endpoint.
    #[must_use]
    pub fn builder() -> EmailBuilder {
        EmailBuilder::default()
    }
}
