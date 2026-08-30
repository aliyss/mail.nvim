use derive_builder::Builder;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MailProviderType {
    #[default]
    Himalaya,
    /// A provider that serves deterministic fake data through the same async
    /// pipeline as the real one, used by the tests (and handy for demos).
    Fake,
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
            MailProviderType::Fake => "fake",
        };

        id.to_string()
    }

    fn mail_provider_location_default(&self) -> Result<PathBuf, String> {
        let path = ProjectDirs::from("com", "pimalaya", "himalaya")
            .ok_or_else(|| "failed to get configuration directory".to_owned())?
            .config_dir()
            .to_owned();

        // The fake provider never reads a configuration file, so it does not
        // need the himalaya directory to exist.
        if matches!(self.provider_type, MailProviderType::Fake) {
            return Ok(path);
        }

        if !path.exists() {
            // TODO: Create the directory and start the himalaya configuration wizard.
            Err(format!("expected path to exist: {:#}", path.display()))
        } else if !path.is_dir() {
            Err(format!("expected path to directory: {:#}", path.display()))
        } else {
            Ok(path)
        }
    }

    #[expect(
        clippy::unused_self,
        reason = "this pattern is recommended by the derive_builder documentation"
    )]
    fn mail_provider_location_file_name(&self) -> PathBuf {
        PathBuf::from("config.toml")
    }
}
