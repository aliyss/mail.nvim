//! This module contains the global configuration options for the application.

mod email;

pub use email::{
    Email, EmailBuilder, EmailBuilderError, Format, ViewAs, ViewAsBuilder, ViewAsBuilderError,
};
use serde::{Deserialize, Serialize};

use std::path::PathBuf;

use derive_builder::Builder;
use directories::ProjectDirs;

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

    /// Save the configuration to a json file.
    /// # Errors
    /// Returns an error if the file cannot be written.
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
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
        println!("{config:?}");
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
        println!("{config:?}");
    }
}
