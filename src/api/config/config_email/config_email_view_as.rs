use crate::api::api_prelude::ViewAsFormat;
use derive_builder::Builder;

/// ViewAs configuration options.
#[derive(Debug, Builder, Clone)]
#[builder(setter(strip_option))]
pub struct ConfigEmailViewAs<'a> {
    /// Format of the output.
    #[builder(setter(into), default = "ViewAsFormat::Plain")]
    format: ViewAsFormat,

    /// Command to execute.
    #[builder(setter(into), default)]
    command: Option<&'a str>,

    /// Should capture output.
    #[builder(setter(into), default)]
    capture_output: Option<bool>,
}

impl<'a> ConfigEmailViewAs<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> ConfigEmailViewAsBuilder<'a> {
        ConfigEmailViewAsBuilder::default()
    }
}

impl<'a> ConfigEmailViewAsBuilder<'a> {}
