use derive_builder::Builder;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MailProviderType {
    #[default]
    Himalaya,
}

/// `Email` configuration options.
#[derive(Debug, Clone, PartialEq, Eq, Builder, Serialize, Deserialize)]
#[builder(setter(strip_option))]
pub struct MailProvider {
    #[builder(setter(into), default = "self.mail_provider_id_default()")]
    pub id: String,

    /// Location of the setting to be set to.
    #[builder(default = "self.mail_provider_location_default()?")]
    pub location: PathBuf,

    /// Location of the setting to be set to.
    #[builder(default = "self.mail_provider_location_file_name()")]
    pub file_name: PathBuf,

    /// Type of mail provider.
    #[builder(setter(into), field(ty = "MailProviderType"))]
    pub provider_type: MailProviderType,
}

impl MailProvider {
    /// Create a builder for the endpoint.
    #[must_use]
    pub fn builder() -> MailProviderBuilder {
        MailProviderBuilder::default()
    }
}

impl MailProviderBuilder {
    fn mail_provider_id_default(&self) -> String {
        let id = match self.provider_type {
            MailProviderType::Himalaya => "himalaya",
        };

        id.to_string()
    }

    fn mail_provider_location_default(&self) -> Result<PathBuf, String> {
        let project_dirs = match self.provider_type {
            MailProviderType::Himalaya => ProjectDirs::from("com", "pimalaya", "himalaya"),
            // MailProviderType::Other => ProjectDirs::from("foo", "bar", "baz"),
        }
        .ok_or_else(|| "failed to get configuration directory".to_owned())?;

        let path = project_dirs.config_dir();

        if !path.exists() {
            // TODO: Create the directory and start the himalaya configuration wizard.
            Err(format!("expected path to exist: {:#}", path.display()))
        } else if !path.is_dir() {
            Err(format!("expected path to directory: {:#}", path.display()))
        } else {
            Ok(path.to_owned())
        }
    }

    fn mail_provider_location_file_name(&self) -> PathBuf {
        let file_name = match self.provider_type {
            MailProviderType::Himalaya => "config.toml",
        };

        PathBuf::from(file_name)
    }
}
