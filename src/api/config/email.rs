use std::collections::HashMap;

use derive_builder::Builder;

/// `ViewAs` configuration options.
#[derive(Debug, Clone, Builder)]
#[builder(setter(strip_option))]
pub struct Email<'a> {
    /// `ViewAs` command configuration.
    #[builder(setter(into), default)]
    view_as_commands: Option<&'a HashMap<String, ViewAs<'a>>>,

    /// Default `view_as` command.
    #[builder(setter(into), default)]
    view_as_commands_default: Option<&'a str>,
}

impl<'a> Email<'a> {
    /// Create a builder for the endpoint.
    #[must_use]
    pub fn builder() -> EmailBuilder<'a> {
        EmailBuilder::default()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Format {
    #[default]
    Plain,
    Json,
    Html,
}

/// `ViewAs` configuration options.
#[derive(Debug, Builder, Clone)]
#[builder(setter(strip_option))]
pub struct ViewAs<'a> {
    /// Format of the output.
    #[builder(setter(into))]
    format: Format,

    /// Command to execute.
    #[builder(setter(into), default)]
    command: Option<&'a str>,

    /// Should capture output.
    #[builder(setter(into), default)]
    capture_output: Option<bool>,
}

impl<'a> ViewAs<'a> {
    /// Create a builder for the endpoint.
    #[must_use]
    pub fn builder() -> ViewAsBuilder<'a> {
        ViewAsBuilder::default()
    }
}
