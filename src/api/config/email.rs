use std::collections::HashMap;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

/// Represents the file format that the email will be displayed in.
#[derive(Debug, Clone, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Format {
    #[default]
    Plain,
    Json,
    Html,
}

/// Represents the email configuration options.
#[derive(Debug, Clone, PartialEq, Eq, Builder, Serialize, Deserialize)]
#[builder(setter(strip_option))]
pub struct Email {
    #[builder(setter(into), default)]
    view_as_commands: HashMap<Format, ViewAs>,

    #[builder(setter(into), default)]
    view_as_commands_default: Option<Format>,
}

impl Email {
    #[must_use]
    pub fn builder() -> EmailBuilder {
        EmailBuilder::default()
    }
}

/// Represents the view configuration options.
#[derive(Debug, Builder, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[builder(setter(strip_option))]
pub struct ViewAs {
    /// The shell command to execute.
    #[builder(setter(into), default)]
    command: String,

    /// Whether to save the command's output.
    #[builder(setter(into), default = "self.capture_output_default()")]
    capture_output: bool,
}

impl ViewAs {
    /// Create a builder for the endpoint.
    #[must_use]
    pub fn builder() -> ViewAsBuilder {
        ViewAsBuilder::default()
    }
}

impl ViewAsBuilder {
    #[expect(
        clippy::unused_self,
        reason = "this pattern is recommended by the derive_builder documentation"
    )]
    fn capture_output_default(&self) -> bool {
        true
    }
}
