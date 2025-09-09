pub mod account;

use crate::api::config::Config;
use std::path::PathBuf;

use derive_builder::Builder;
use himalaya::config::TomlConfig;
use pimalaya_tui::terminal::config::TomlConfig as _;
use serde::{Deserialize, Serialize};

/// `Email` configuration options.
#[derive(Debug, Clone, PartialEq, Eq, Builder, Serialize, Deserialize)]
#[builder(setter(strip_option))]
pub struct HimalayaProvider {
    /// Configuration of Himalaya
    #[builder(setter(into))]
    pub config: TomlConfig,
}

impl HimalayaProvider {
    /// Create a builder for the endpoint.
    #[must_use]
    pub fn builder() -> HimalayaProviderBuilder {
        HimalayaProviderBuilder::default()
    }
}

impl TryFrom<PathBuf> for HimalayaProvider {
    type Error = HimalayaProviderBuilderError;
    fn try_from(config_path: PathBuf) -> Result<Self, Self::Error> {
        let config = TomlConfig::from_paths(&[config_path]).map_err(|err| {
            HimalayaProviderBuilderError::ValidationError(format!(
                "failed to load config from path: {err}"
            ))
        })?;
        Ok(HimalayaProvider { config })
    }
}

impl TryFrom<Config> for HimalayaProvider {
    type Error = HimalayaProviderBuilderError;
    fn try_from(config: Config) -> Result<Self, Self::Error> {
        HimalayaProvider::try_from(
            config
                .mail_provider
                .location
                .join(config.mail_provider.file_name),
        )
    }
}

#[cfg(test)]
mod tests {

    use pimalaya_tui::terminal::config::TomlConfig;

    use super::*;
    use crate::api::config::Config;

    #[test]
    fn himalaya_provider() {
        let config = Config::builder()
            .build()
            .expect("Expected default builder to be valid");
        let himalaya_provider =
            HimalayaProvider::try_from(config.clone()).expect("failed to create himalaya provider");
        let default_config = TomlConfig::from_paths(&[config
            .mail_provider
            .location
            .join(config.mail_provider.file_name)])
        .expect("failed to load default himalaya config from path");

        assert_eq!(
            himalaya_provider.config, default_config,
            "Expected default himalaya config to be equal"
        );
    }
}
