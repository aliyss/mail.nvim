use std::future::Future;

use crate::api::account::Account;
use crate::api::folder::Folder;

pub trait ListFolders {
    /// Execute the list command using the provided mail provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn list_folders(
        &self,
        account: &Account,
    ) -> impl Future<Output = anyhow::Result<Vec<Folder>>> + Send;
}

pub trait GetFolder {
    /// Execute the get command using the provided mail provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn get_folder(
        &self,
        account: &Account,
        folder_id: &str,
    ) -> impl Future<Output = anyhow::Result<Option<Folder>>> + Send;
}

pub trait CreateFolder {
    /// Execute the create command using the provided mail provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn create_folder(
        &self,
        account: &Account,
        folder_name: &str,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait DeleteFolder {
    /// Execute the delete command using the provided mail provider.
    /// This command removes the specified folder from the mail account including all its messages.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn delete_folder(
        &self,
        account: &Account,
        folder_id: &str,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait ExpungeMessages {
    /// Execute the expunge command using the provided mail provider.
    /// This command removes all messages marked as deleted in the specified folder.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn expunge_messages(
        &self,
        account: &Account,
        folder_id: &str,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait PurgeMessages {
    /// Execute the purge command using the provided mail provider.
    /// This operation removes all messages in the specified folder, regardless of their flags.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn purge_messages(
        &self,
        account: &Account,
        folder_id: &str,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}
