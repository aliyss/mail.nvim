//! This module contains the global configuration options for the application.

mod email;

pub use email::{
    Email, EmailBuilder, EmailBuilderError, Format, ViewAs, ViewAsBuilder, ViewAsBuilderError,
};
use serde::{Deserialize, Serialize};

use std::path::PathBuf;

use derive_builder::Builder;
use directories::ProjectDirs;

use crate::api::file::TryFile;

/// Configuration for all settings within the Mailbox.
#[derive(Debug, Builder, Clone, Serialize, Deserialize)]
#[builder(setter(strip_option))]
pub struct Config {
    /// Location of the setting to be set to.
    #[builder(default = "ConfigBuilder::set_mail_provider_location_default()?")]
    mail_provider_location: PathBuf,

    /// Email config
    #[builder(setter(into, strip_option), default)]
    email: Option<Email>,

    /// `UserHandholding`
    #[builder(setter(into, strip_option), default)]
    user_handholding: Option<bool>,
    /// `UserHandHandholding`
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

impl TryFile for Config {
    fn file_name() -> String {
        "config.json".to_owned()
    }

    fn try_default() -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        Config::builder()
            .build()
            .map_err(|e| std::io::Error::other(format!("Failed to build default Config: {e}")))
    }
}

impl ConfigBuilder {
    fn set_mail_provider_location_default() -> Result<PathBuf, String> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "pimalaya", "himalaya") {
            let config_dir = proj_dirs.config_dir();
            let path = config_dir.to_path_buf();
            if !path.exists() {
                // TODO: This should automatically create the directory and start the himalaya
                // configuration wizard.
                return Err(
                    "Configuration directory does not exist at {path:?}. Please create it first."
                        .to_string(),
                );
            }
            Ok(config_dir.to_path_buf())
        } else {
            Err("Could not determine configuration directory".to_owned())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn config_builder() {
        let config = Config::builder()
            .build()
            .expect("Expected default builder to be valid");
        assert!(
            config.mail_provider_location
                == ConfigBuilder::set_mail_provider_location_default().unwrap()
        );
        assert!(config.email.is_none());
        assert!(config.user_handholding.is_none());
        assert!(config.user_handhandholding.is_none());
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
                    .unwrap(),
            ),
            (
                "html".to_owned(),
                ViewAs::builder()
                    .format(Format::Html)
                    .command("w3m -T text/html")
                    .capture_output(true)
                    .build()
                    .unwrap(),
            ),
            (
                "plain".to_owned(),
                ViewAs::builder()
                    .format(Format::Plain)
                    .command("cat")
                    .capture_output(true)
                    .build()
                    .unwrap(),
            ),
        ]);
        let email_config = Email::builder().view_as_commands(binding).build().unwrap();
        let config = Config::builder().email(email_config).build().unwrap();
        assert!(config.email.is_some());
    }

    #[test]
    fn config_from_default_path() {
        let config = Config::read_file(None);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert!(
            config.mail_provider_location
                == ConfigBuilder::set_mail_provider_location_default().unwrap()
        );
    }

    #[test]
    fn config_from_invalid_path() {
        let config = Config::read_file(Some(PathBuf::from("/invalid/path/")));
        println!("invalid path test result: {config:?}");
        assert!(config.is_err());
    }
}
