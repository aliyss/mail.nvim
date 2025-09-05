//! This module contains the global configuration options for the application.

mod email;

pub use email::{
    Email, EmailBuilder, EmailBuilderError, Format, ViewAs, ViewAsBuilder, ViewAsBuilderError,
};

use std::path::PathBuf;

use derive_builder::Builder;
use directories::ProjectDirs;

/// Configuration for all settings within the Mailbox.
#[derive(Debug, Builder, Clone)]
#[builder(setter(strip_option))]
pub struct Config<'a> {
    /// Location of the setting to be set to.
    #[builder(
        setter(name = "himalaya_location"),
        default = "ConfigBuilder::set_himalaya_location_default()?"
    )]
    himalaya_location: PathBuf,

    /// Email config
    #[builder(setter(into, strip_option), default)]
    email: Option<&'a Email<'a>>,
    /// `UserHandholding`
    #[builder(setter(into, strip_option), default)]
    user_handholding: Option<bool>,

    /// `UserHandHandholding`
    #[builder(setter(into, strip_option), default)]
    user_handhandholding: Option<bool>,
}

impl<'a> Config<'a> {
    /// Create a builder for the endpoint.
    #[must_use]
    pub fn builder() -> ConfigBuilder<'a> {
        ConfigBuilder::default()
    }
}

impl ConfigBuilder<'_> {
    fn set_himalaya_location_default() -> Result<PathBuf, String> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "pimalaya", "himalaya") {
            let config_dir = proj_dirs.config_dir();
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
            .expect("expected default builder to be valid");
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
        let email_config = Email::builder().view_as_commands(&binding).build().unwrap();
        let config = Config::builder().email(&email_config).build().unwrap();
        println!("{config:?}");
    }
}
