mod account;
mod envelope;
mod folder;

use crate::api::config::Config;
use std::path::PathBuf;

use derive_builder::Builder;
use pimalaya_tui::himalaya::config::HimalayaTomlConfig;
use pimalaya_tui::terminal::config::TomlConfig as _;
use serde::{Deserialize, Serialize};

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
