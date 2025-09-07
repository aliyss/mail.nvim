// TODO: Make sure that nvim print statements can be uncommented and still have tests compiled
use serde::Serialize;

use directories::ProjectDirs;
// use nvim_oxi as nvim;

use crate::constants::{MAIL_APPLICATION, MAIL_ORGANIZATION, MAIL_QUALIFIER};

use serde::de::DeserializeOwned;
use std::path::PathBuf;

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

pub trait TryFile<Error = std::io::Error>
where
    Self: Sized,
    Error: std::fmt::Display
        + std::fmt::Debug
        + From<std::io::Error>
        + std::marker::Send
        + std::marker::Sync
        + std::error::Error
        + 'static,
{
    fn file_name() -> String;

    /// Attempts to create a default instance of the implementing type.
    /// # Errors
    /// Returns an error if the default instance cannot be created.
    fn try_default() -> Result<Self, Error>
    where
        Self: Sized;

    /// Saves the current instance to a file at the specified path.
    /// # Errors
    /// Returns an error if the instance cannot be serialized or written to the file.
    fn save_to_file(&self, path: &PathBuf) -> Result<(), Error>
    where
        Self: Serialize,
    {
        let serialized = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::other(format!("Serialization error: {e}")))?;
        std::fs::write(path, serialized)?;
        Ok(())
    }

    /// Reads an instance of the implementing type from a file at the specified path.
    /// # Errors
    /// Returns an error if the file cannot be read or the instance cannot be deserialized.
    fn read_file(value: Option<PathBuf>) -> Result<Self, Error>
    where
        Self: Sized + DeserializeOwned + Serialize,
    {
        let default_config_location = setup_config_local_dir()?;
        let file_name = Self::file_name();

        let config_location = match &value {
            Some(path) => path.join(file_name),
            None => default_config_location.join(file_name),
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
                    return Err(err.into());
                }

                // If the file does not exist, create a default config file
                let config = match Self::try_default() {
                    Ok(cfg) => cfg,
                    Err(err) => {
                        // nvim::print!("[{MAIL_PLUGIN_NAME}] Failed to create default config: {err}");
                        return Err(err);
                    }
                };
                if !config_location.exists() {
                    std::fs::create_dir_all(&config_location)?;
                }
                let _ = config.save_to_file(&config_location);
                return Ok(config);
            }
        };

        let config: Self = match serde_json::from_str(&config_data) {
            Ok(cfg) => cfg,
            Err(err) => {
                // nvim::print!(
                //     "[{MAIL_PLUGIN_NAME}] Failed to parse config file at {config_location:?}: {err}"
                // );
                return Err(
                    std::io::Error::other(format!("Failed to parse config file: {err}")).into(),
                );
            }
        };

        Ok(config)
    }
}
