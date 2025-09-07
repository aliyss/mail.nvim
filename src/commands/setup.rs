// TODO: Make sure that nvim print statements can be uncommented and still have tests compiled
use std::path::PathBuf;

use directories::ProjectDirs;
// use nvim_oxi as nvim;

use crate::{
    api::config::Config,
    constants::{MAIL_APPLICATION, MAIL_ORGANIZATION, MAIL_QUALIFIER},
};

/// Sets up the local data directory for the application.
/// Returns the path to the local data directory.
/// # Errors
/// Returns an error if the directory cannot be created or determined.
pub fn setup_config_local_dir() -> Result<PathBuf, std::io::Error> {
    if let Some(proj_dirs) = ProjectDirs::from(MAIL_QUALIFIER, MAIL_ORGANIZATION, MAIL_APPLICATION)
    {
        let config_dir = proj_dirs.data_local_dir();
        let path = config_dir.to_path_buf();
        if !path.exists() {
            // nvim::print!("[{MAIL_PLUGIN_NAME}] Creating local data directory at {path:?}");
            std::fs::create_dir_all(&path)?;
        }
        Ok(path)
    } else {
        Err(std::io::Error::other(
            "Could not determine local data directory",
        ))
    }
}

impl TryFrom<Option<PathBuf>> for Config {
    type Error = std::io::Error;

    fn try_from(value: Option<PathBuf>) -> Result<Self, std::io::Error> {
        let default_config_location = setup_config_local_dir()?;

        let config_location = match &value {
            Some(path) => path.join("config.json"),
            None => default_config_location.join("config.json"),
        };

        let config_data = match std::fs::read_to_string(&config_location) {
            Ok(data) => data,
            Err(err) => {
                // nvim::print!(
                //     "[{MAIL_PLUGIN_NAME}] Failed to read config file at {config_location:?}: {err}"
                // );

                // Throw error if either of the following conditions are met:
                // 1. The config location is not the default config location
                // 2. The file exists but is not readable for some reason
                if config_location != default_config_location.join("config.json")
                    || err.kind() != std::io::ErrorKind::NotFound
                {
                    return Err(err);
                }

                // If the file does not exist, create a default config file
                let config = match Config::builder().build() {
                    Ok(cfg) => cfg,
                    Err(err) => {
                        // nvim::print!("[{MAIL_PLUGIN_NAME}] Failed to create default config: {err}");
                        return Err(std::io::Error::other(err));
                    }
                };
                if !default_config_location.exists() {
                    std::fs::create_dir_all(default_config_location)?;
                }
                let _ = config.save_to_file(&config_location);
                return Ok(config);
            }
        };

        let config: Config = match serde_json::from_str(&config_data) {
            Ok(cfg) => cfg,
            Err(err) => {
                // nvim::print!(
                //     "[{MAIL_PLUGIN_NAME}] Failed to parse config file at {config_location:?}: {err}"
                // );
                return Err(err.into());
            }
        };

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use crate::api::config::Config;
    use std::path::PathBuf;

    #[test]
    fn test_config_try_from_none() {
        let config = Config::try_from(None);
        assert!(config.is_ok());
        let config = config.unwrap();
        println!("{config:?}");
    }

    #[test]
    fn test_config_try_from_invalid_path() {
        let config = Config::try_from(Some(PathBuf::from("/invalid/path/config.json")));
        println!("invalid path test result: {config:?}");
        assert!(config.is_err());
        println!("{:?}", config.err());
    }
}
