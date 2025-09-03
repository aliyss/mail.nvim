mod config_email_view_as;

use std::collections::HashMap;

use derive_builder::Builder;

use crate::api::config::config_email::config_email_view_as::ConfigEmailViewAs;

/// ViewAs configuration options.
#[derive(Debug, Builder, Clone)]
#[builder(setter(strip_option))]
pub struct ConfigEmail<'a> {
    /// ViewAs command configuration.
    #[builder(setter(into), default)]
    view_as_commands: Option<&'a HashMap<String, ConfigEmailViewAs<'a>>>,

    /// Default view_as command.
    #[builder(setter(into), default)]
    view_as_commands_default: Option<&'a str>,
}

impl<'a> ConfigEmail<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> ConfigEmailBuilder<'a> {
        ConfigEmailBuilder::default()
    }
}

impl<'a> ConfigEmailBuilder<'a> {}
