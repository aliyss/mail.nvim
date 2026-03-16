mod account;
mod envelope;
mod folder;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::anyhow;
use derive_builder::Builder;
use email::account::config::AccountConfig as EmailAccountConfig;
use email::backend::BackendBuilder as EmailBackendBuilder;
use email::config::Config as EmailConfig;
use pimalaya_tui::himalaya::backend::{Backend, BackendBuilder, ContextBuilder};
use pimalaya_tui::himalaya::config::{HimalayaTomlAccountConfig, HimalayaTomlConfig};
use pimalaya_tui::terminal::config::TomlConfig as _;
use serde::{Deserialize, Serialize};

use crate::api::config::Config;

/// `Email` configuration options.
#[derive(Debug, Clone, PartialEq, Eq, Builder, Serialize, Deserialize)]
#[builder(setter(strip_option))]
pub struct HimalayaProvider {
    /// Configuration of Himalaya
    #[builder(setter(into))]
    config: HimalayaTomlConfig,
}

impl HimalayaProvider {
    /// Create a builder for the endpoint.
    #[must_use]
    pub fn builder() -> HimalayaProviderBuilder {
        HimalayaProviderBuilder::default()
    }

    /// Construct a new provider from a file at the given path.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - We are unable to read the configuration file (e.g., it does not exist).
    /// - There is an issue parsing the configuration.
    pub fn from_path(path: PathBuf) -> Result<Self, HimalayaProviderBuilderError> {
        let config = HimalayaTomlConfig::from_paths(&[path]).map_err(|err| {
            HimalayaProviderBuilderError::ValidationError(format!(
                "failed to load config from path: {err}"
            ))
        })?;

        Ok(HimalayaProvider { config })
    }

    /// Construct a new provider from an existing configuration.
    ///
    /// This is a convenience method for calling [`Self::from_path`] by constructing the path
    /// with the information provided in [`crate::api::config::Config`].
    ///
    /// # Errors
    ///
    /// Refer to [`Self::from_path`].
    pub fn from_config(config: &Config) -> Result<Self, HimalayaProviderBuilderError> {
        let Config { mail_provider, .. } = config;
        HimalayaProvider::from_path(mail_provider.location.join(&mail_provider.file_name))
    }

    #[must_use]
    pub fn config(&self) -> &HimalayaTomlConfig {
        &self.config
    }

    #[expect(clippy::missing_errors_doc, reason = "todo")]
    pub fn get_account_config(
        &self,
        account_name: &str,
    ) -> Result<(HimalayaTomlAccountConfig, EmailAccountConfig), anyhow::Error> {
        let (account_info_name, himalaya_config) = self
            .config
            .get_account_config(account_name)
            .ok_or_else(|| anyhow!("failed to get Himalaya account config"))?;

        let email_config = EmailConfig::from(self.config.clone())
            .account(account_info_name)
            .map_err(|_| anyhow!("failed to get email config"))?;

        Ok((himalaya_config, email_config))
    }

    #[expect(clippy::missing_errors_doc, reason = "todo")]
    pub async fn get_backend_from_config<F>(
        himalaya_config: HimalayaTomlAccountConfig,
        email_config: EmailAccountConfig,
        func: F,
    ) -> anyhow::Result<Backend>
    where
        F: Fn(EmailBackendBuilder<ContextBuilder>) -> EmailBackendBuilder<ContextBuilder>,
    {
        BackendBuilder::new(Arc::new(himalaya_config), Arc::new(email_config), func)
            .without_sending_backend()
            .build()
            .await
            .map_err(|_| anyhow!("failed to build backend"))
    }

    #[expect(clippy::missing_errors_doc, reason = "todo")]
    pub async fn get_backend<F>(&self, account_name: &str, func: F) -> anyhow::Result<Backend>
    where
        F: Fn(EmailBackendBuilder<ContextBuilder>) -> EmailBackendBuilder<ContextBuilder>,
    {
        let (himalaya_config, email_config) = self.get_account_config(account_name)?;
        Self::get_backend_from_config(himalaya_config, email_config, func).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn himalaya_provider() {
        let config = Config::builder()
            .build()
            .expect("Expected default builder to be valid");
        let himalaya_provider =
            HimalayaProvider::from_config(&config).expect("failed to create himalaya provider");
        let path = config
            .mail_provider
            .location
            .join(config.mail_provider.file_name);
        let default_config = HimalayaTomlConfig::from_paths(&[path])
            .expect("failed to load default himalaya config from path");

        assert_eq!(
            himalaya_provider.config(),
            &default_config,
            "Expected default himalaya config to be equal"
        );
    }
}
