// TODO: Make sure that nvim print statements can be uncommented and still have tests compiled

use std::path::{self, Path, PathBuf};
use std::{error, fs, io};

use directories::ProjectDirs;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::constants::{MAIL_APPLICATION, MAIL_ORGANIZATION, MAIL_QUALIFIER};

/// Prepare the local data directory for the application, returning the path to that directory.
///
/// # Panics
///
/// This function panics if the path returned by [`directories::ProjectDirs::data_local_dir`] is
/// not absolute.
///
/// # Errors
///
/// This function returns an error if:
///
/// - A valid home directory could not be determined.
/// - The expected directories could not be created due to lack of permissions or other errors
///   (see [`std::fs::create_dir_all`] for details).
pub fn prepare_default_data_directory() -> Result<PathBuf, io::Error> {
    match ProjectDirs::from(MAIL_QUALIFIER, MAIL_ORGANIZATION, MAIL_APPLICATION) {
        Some(project_dirs) => {
            let path = project_dirs.data_local_dir().to_owned();
            assert!(path.is_absolute());
            fs::create_dir_all(&path)?;
            Ok(path)
        }
        None => Err(io::Error::other("failed to get home directory")),
    }
}

/// A trait for (de)serializing data to (and from) a file.
pub trait TryFile
where
    Self: Sized + Serialize + DeserializeOwned,
{
    type Error: error::Error + Send + Sync + 'static + From<io::Error>;

    const FILE_NAME: &'static str;

    /// Attempts to create a default instance of the implementing type.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - The default instance cannot be created.
    fn try_default() -> Result<Self, Self::Error>;

    /// Save the current instance to a file at the specified path.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - The instance cannot be serialized into valid JSON.
    /// - Writing the data to the file fails (e.g., due to lack of permissions)
    ///   (see [`std::fs::write`] for details).
    fn write_to_file<P>(&self, path: P) -> Result<(), Self::Error>
    where
        P: AsRef<Path>,
    {
        let serialized = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::other(format!("Serialization error: {e}")))?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Construct an instance of `Self` from the given path.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - We are unable to get the current directory.
    /// - `path` is `Some` and the given path is an empty [`std::path::PathBuf`].
    /// - `path` is `None` and we are unable to get the default configuration directory.
    /// - The configuration file does not exist and we are unable to create the default
    ///   configuration (e.g., due to lack of permissions).
    /// - We are unable to parse the configuration file.
    fn read_from_file(path: Option<PathBuf>) -> Result<Self, Self::Error> {
        // TODO: Currently if the file is missing a field, the default value for that field is not
        // being set.
        let path = if let Some(path) = path {
            path::absolute(&path)?
        } else {
            let directory_default = prepare_default_data_directory()?;
            assert!(directory_default.is_absolute());
            directory_default.join(Self::FILE_NAME)
        };

        match fs::read_to_string(&path) {
            Ok(data) => serde_json::from_str(&data).map_err(|err| {
                io::Error::other(format!("failed to parse configuration file: {err}")).into()
            }),
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                tracing::info!("writing default configuration to: {:#}", path.display());
                fs::create_dir_all(path.parent().expect("expected path to be absolute"))?;
                // XXX(Nic): Under what conditions can this fail? If we have control over the
                // default configuration, can we change this to `std::default::Default`?
                let config = Self::try_default()?;
                config.write_to_file(&path)?;
                Ok(config)
            }
            Err(err) => Err(err.into()),
        }
    }
}
