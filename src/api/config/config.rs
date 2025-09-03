use std::path::PathBuf;

use directories::ProjectDirs;

use derive_builder::Builder;

use crate::api::config::config_email::ConfigEmail;

/// Configuration for all settings within the Mailbox.
#[derive(Debug, Builder, Clone)]
#[builder(setter(strip_option))]
pub struct Config<'a> {
    /// Location of the setting to be set to.
    #[builder(
        setter(name = "himalaya_location"),
        default = "self.set_himalaya_location_default()?"
    )]
    himalaya_location: PathBuf,

    /// Email config
    #[builder(setter(into, strip_option), default)]
    email: Option<&'a ConfigEmail<'a>>,
}

impl<'a> Config<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> ConfigBuilder<'a> {
        ConfigBuilder::default()
    }
}

impl<'a> ConfigBuilder<'a> {
    fn set_himalaya_location_default(&self) -> Result<PathBuf, String> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "pimalaya", "himalaya") {
            let config_dir = proj_dirs.config_dir();
            Ok(config_dir.to_path_buf())
        } else {
            Err("Could not determine configuration directory".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = Config::builder().build().unwrap();
        println!("{config:?}");
    }
}
