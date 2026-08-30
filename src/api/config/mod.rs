//! This module contains the global configuration options for the application.

mod email;
mod provider;
pub mod ui;

pub use email::{
    Email, EmailBuilder, EmailBuilderError, Format, ViewAs, ViewAsBuilder, ViewAsBuilderError,
};
pub use provider::{MailProvider, MailProviderBuilder, MailProviderBuilderError, MailProviderType};

use std::io;

use crate::{
    api::file::TryFile,
    providers::{AnyProvider, FakeProvider, HimalayaProvider, Provider},
};

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

    /// Default path for UI views.
    #[builder(
        setter(into, strip_option),
        default = "self.default_view_path_default()"
    )]
    pub default_view_path: String,

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

    /// Whether risky (`!`) actions require confirmation before running.
    ///
    /// Enabled by default; disable with
    /// `:MailConfigUserHandHoldingSwitchOn false`.
    #[must_use]
    pub fn user_handholding(&self) -> bool {
        self.user_handholding.unwrap_or(true)
    }

    /// Whether extra risky (`!!`) actions require confirmation before running.
    ///
    /// Enabled by default; disable with
    /// `:MailConfigUserHandHandHoldingSwitchOn false`.
    #[must_use]
    pub fn user_handhandholding(&self) -> bool {
        self.user_handhandholding.unwrap_or(true)
    }

    /// Enables or disables confirmation for risky (`!`) actions.
    pub fn set_user_handholding(&mut self, enabled: bool) {
        self.user_handholding = Some(enabled);
    }

    /// Enables or disables confirmation for extra risky (`!!`) actions.
    pub fn set_user_handhandholding(&mut self, enabled: bool) {
        self.user_handhandholding = Some(enabled);
    }

    /// Create a provider from the configuration.
    ///
    /// # Errors
    /// Returns an error if the provider could not be created.
    pub fn to_provider(&self) -> Result<impl Provider, anyhow::Error> {
        match self.mail_provider.provider_type {
            MailProviderType::Himalaya => Ok(AnyProvider::Himalaya(HimalayaProvider::from_config(
                self,
            )?)),
            MailProviderType::Fake => Ok(AnyProvider::Fake(FakeProvider)),
        }
    }
}

impl ConfigBuilder {
    #[expect(
        clippy::unused_self,
        reason = "this pattern is recommended by the derive_builder documentation"
    )]
    fn mail_provider_default(&self) -> Result<MailProvider, ConfigBuilderError> {
        MailProvider::builder().build().map_err(|_err| {
            ConfigBuilderError::UninitializedField(
                "failed to create/get default mail provider location",
            )
        })
    }
    #[expect(
        clippy::unused_self,
        reason = "this pattern is recommended by the derive_builder documentation"
    )]
    fn default_view_path_default(&self) -> String {
        "default.json".into()
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
                Format::Json,
                ViewAs::builder()
                    .command("jq .")
                    .capture_output(true)
                    .build()
                    .expect("expected hard-coded ViewAs format to be valid"),
            ),
            (
                Format::Html,
                ViewAs::builder()
                    .command("w3m -T text/html")
                    .capture_output(true)
                    .build()
                    .expect("expected hard-coded ViewAs format to be valid"),
            ),
            (
                Format::Plain,
                ViewAs::builder()
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
    fn user_handholding_defaults_to_enabled() {
        let config = Config::builder()
            .build()
            .expect("expected default builder to be valid");

        assert!(config.user_handholding());
        assert!(config.user_handhandholding());
    }

    #[test]
    fn user_handholding_can_be_disabled() {
        let mut config = Config::builder()
            .build()
            .expect("expected default builder to be valid");

        config.set_user_handholding(false);
        config.set_user_handhandholding(false);

        assert!(!config.user_handholding());
        assert!(!config.user_handhandholding());
    }

    #[test]
    fn user_handholding_survives_round_trip() {
        let mut config = Config::builder()
            .build()
            .expect("expected default builder to be valid");
        config.set_user_handholding(false);

        let json = serde_json::to_string(&config).expect("config should serialize");
        let decoded: Config = serde_json::from_str(&json).expect("config should parse");

        assert!(!decoded.user_handholding());
        assert!(decoded.user_handhandholding());
    }

    #[test]
    fn fake_provider_type_builds_the_fake_provider() {
        use crate::api::account::commands::ListAccounts;

        let config = Config::builder()
            .mail_provider(
                MailProvider::builder()
                    .provider_type(MailProviderType::Fake)
                    .build()
                    .expect("expected fake mail provider to be valid"),
            )
            .build()
            .expect("expected fake configuration to be valid");

        let provider = config
            .to_provider()
            .expect("expected the fake provider to be created");

        let accounts = provider
            .list_accounts()
            .expect("expected the fake accounts to list");
        assert!(
            accounts.iter().any(|account| account.name() == "nic@example.com"),
            "expected the fake provider to serve its fake accounts"
        );
    }

    #[test]
    fn fake_provider_type_survives_a_round_trip() {
        let config = Config::builder()
            .mail_provider(
                MailProvider::builder()
                    .provider_type(MailProviderType::Fake)
                    .build()
                    .expect("expected fake mail provider to be valid"),
            )
            .build()
            .expect("expected fake configuration to be valid");

        let json = serde_json::to_string(&config).expect("config should serialize");
        let decoded: Config = serde_json::from_str(&json).expect("config should parse");
        assert_eq!(
            decoded.mail_provider.provider_type,
            MailProviderType::Fake
        );
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
