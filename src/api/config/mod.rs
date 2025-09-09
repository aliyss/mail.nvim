//! This module contains the global configuration options for the application.

mod email;
mod provider;

pub use email::{
    Email, EmailBuilder, EmailBuilderError, Format, ViewAs, ViewAsBuilder, ViewAsBuilderError,
};
pub use provider::{MailProvider, MailProviderBuilder, MailProviderBuilderError};

use std::io;

use crate::api::file::TryFile;

/// Configuration for all settings within the Mailbox.
#[derive(Debug, Clone, derive_builder::Builder, serde::Serialize, serde::Deserialize)]
#[builder(setter(strip_option))]
pub struct Config {
    /// Location of the setting to be set to.
    #[builder(default = "self.mail_provider_default()?")]
    pub mail_provider: MailProvider,

    /// Email config
    #[builder(setter(into, strip_option), default)]
    email: Option<Email>,

    /// Risky actions require confirmation.
    #[builder(setter(into, strip_option), default)]
    user_handholding: Option<bool>,

    /// Extra risky actions require confirmation.
    #[builder(setter(into, strip_option), default)]
    user_handhandholding: Option<bool>,
}

impl Config {
    /// Create a builder for the endpoint.
    #[must_use]
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

impl ConfigBuilder {
    #[expect(
        clippy::unused_self,
        reason = "this pattern is recommended by the derive_builder documentation"
    )]
    fn mail_provider_default(&self) -> Result<MailProvider, ConfigBuilderError> {
        // TODO (Nic): Better mapping on errors.
        MailProvider::builder().build().map_err(|_err| {
            ConfigBuilderError::UninitializedField(
                "failed to create/get default mail provider location",
            )
        })
    }
}

impl TryFile for Config {
    type Error = io::Error;

    const FILE_NAME: &'static str = "config.json";

    fn try_default() -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Config::builder().build().map_err(|err| {
            io::Error::other(format!("failed to build default configuration: {err}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn config_builder() {
        let config = Config::builder()
            .build()
            .expect("Expected default builder to be valid");

        assert_eq!(
            config.mail_provider.location,
            MailProvider::builder()
                .build()
                .expect("failed to create/get default mail provider location")
                .location
        );
        assert_eq!(config.email, None);
        assert_eq!(config.user_handholding, None);
        assert_eq!(config.user_handhandholding, None);
    }

    #[test]
    fn config_builder_with_email_config() {
        let binding = HashMap::from([
            (
                "json".to_owned(),
                ViewAs::builder()
                    .format(Format::Json)
                    .command("jq .")
                    .capture_output(true)
                    .build()
                    .expect("expected hard-coded ViewAs format to be valid"),
            ),
            (
                "html".to_owned(),
                ViewAs::builder()
                    .format(Format::Html)
                    .command("w3m -T text/html")
                    .capture_output(true)
                    .build()
                    .expect("expected hard-coded ViewAs format to be valid"),
            ),
            (
                "plain".to_owned(),
                ViewAs::builder()
                    .format(Format::Plain)
                    .command("cat")
                    .capture_output(true)
                    .build()
                    .expect("expected hard-coded ViewAs format to be valid"),
            ),
        ]);

        let email = Email::builder()
            .view_as_commands(binding)
            .build()
            .expect("expected hard-coded email configuration to be valid");

        let config = Config::builder()
            .email(email.clone())
            .build()
            .expect("expected hard-coded configuration to be valid");

        assert_eq!(config.email, Some(email));
    }

    #[test]
    fn config_from_default_path() {
        let config = Config::read_from_file(None)
            .expect("expected default configuration to be created automatically");

        assert_eq!(
            config.mail_provider.location,
            MailProvider::builder()
                .build()
                .expect("failed to create/get default mail provider location")
                .location
        );
    }

    #[test]
    fn config_from_invalid_path() {
        Config::read_from_file(Some(PathBuf::from("/invalid/path/to/config.json")))
            .expect_err("expected hard-coded invalid path to fail");
    }
}
