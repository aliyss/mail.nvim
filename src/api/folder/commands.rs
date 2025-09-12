use std::error;
use std::future::Future;

use crate::api::{account::Account, folder::Folder};

pub trait List {
    type Error: error::Error + Send + Sync + 'static;

    /// Execute the list command using the provided mail provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn folders_list(
        &self,
        account: Option<&Account>,
    ) -> impl Future<Output = Result<Vec<Folder>, Self::Error>> + Send;
}

pub trait Get {
    type Error: error::Error + Send + Sync + 'static;

    /// Execute the get command using the provided mail provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn folders_get(
        &self,
        account: Option<&Account>,
        folder_id: &str,
    ) -> impl Future<Output = Result<Folder, Self::Error>> + Send;
}

pub trait Create {
    type Error: error::Error + Send + Sync + 'static;

    /// Execute the get command using the provided mail provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn folders_create(
        &self,
        account: Option<&Account>,
        folder_name: &str,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait Delete {
    type Error: error::Error + Send + Sync + 'static;

    /// Execute the get command using the provided mail provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn folders_delete(
        &self,
        account: Option<&Account>,
        folder_id: &str,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
