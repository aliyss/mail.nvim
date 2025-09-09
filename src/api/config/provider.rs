use derive_builder::Builder;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MailProviderType {
    #[default]
    Himalaya,
}

/// `Email` configuration options.
#[derive(Debug, Clone, PartialEq, Eq, Builder, Serialize, Deserialize)]
#[builder(setter(strip_option))]
pub struct MailProvider {
    /// Location of the setting to be set to.
    #[builder(default = "self.mail_provider_location_default()?")]
    pub location: PathBuf,

    /// Location of the setting to be set to.
    #[builder(default = "self.mail_provider_location_file_name()?")]
    pub file_name: PathBuf,

    /// Type of mail provider.
    #[builder(setter(into), default = MailProviderType::Himalaya)]
    provider_type: MailProviderType,
}

impl MailProviderBuilder {
    fn mail_provider_location_default(&self) -> Result<PathBuf, String> {
        if self.provider_type == Some(MailProviderType::Himalaya) || self.provider_type.is_none() {
            match ProjectDirs::from("com", "pimalaya", "himalaya") {
                Some(project_dirs) => {
                    let path = project_dirs.config_dir().to_owned();

                    if !path.exists() {
                        // TODO: Create the directory and start the himalaya configuration wizard.
                        Err(format!("expected path to exist: {:#}", path.display()))
                    } else if !path.is_dir() {
                        // TODO: Do we need to be able to differentiate between errors?
                        Err(format!("expected path to directory: {:#}", path.display()))
                    } else {
                        Ok(path)
                    }
                }
                None => Err("failed to get configuration directory".to_owned()),
            }
        } else {
            Err("unsupported mail provider type".to_owned())
        }
    }

    fn mail_provider_location_file_name(&self) -> Result<PathBuf, String> {
        if self.provider_type == Some(MailProviderType::Himalaya) || self.provider_type.is_none() {
            Ok(PathBuf::from("config.toml"))
        } else {
            Err("unsupported mail provider type".to_owned())
        }
    }
}

impl MailProvider {
    /// Create a builder for the endpoint.
    #[must_use]
    pub fn builder() -> MailProviderBuilder {
        MailProviderBuilder::default()
    }
}
