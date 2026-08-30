use std::future::Future;

use crate::api::email::arguments::EmailListArguments;
use crate::api::email::{Email, EmailFlag, EmailMessage, ThreadedEmail};

pub trait GetEmail {
    /// Execute the get command using the provided mail provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn get_emails(
        &self,
        account_id: &str,
        email_id: Vec<&str>,
        folder_id: Option<&str>,
        options: Option<EmailListArguments>,
    ) -> impl Future<Output = anyhow::Result<Vec<EmailMessage>>> + Send;
}

pub trait ListEmails {
    /// Execute the list command using the provided mail provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn list_emails(
        &self,
        account_id: &str,
        folder_id: Option<&str>,
        options: Option<EmailListArguments>,
    ) -> impl Future<Output = anyhow::Result<Vec<Email>>> + Send;
}

pub trait ListThreads {
    /// List the emails of the thread the given email belongs to.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn list_threads(
        &self,
        account_id: &str,
        email_id: &str,
        folder_id: Option<&str>,
    ) -> impl Future<Output = anyhow::Result<Vec<ThreadedEmail>>> + Send;
}

pub trait AddEmailFlags {
    /// Add the given flags to the emails matching the given ids.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn add_email_flags(
        &self,
        account_id: &str,
        folder_id: &str,
        email_ids: Vec<&str>,
        flags: Vec<EmailFlag>,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait RemoveEmailFlags {
    /// Remove the given flags from the emails matching the given ids.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn remove_email_flags(
        &self,
        account_id: &str,
        folder_id: &str,
        email_ids: Vec<&str>,
        flags: Vec<EmailFlag>,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait SetEmailFlags {
    /// Replace the flags of the emails matching the given ids.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn set_email_flags(
        &self,
        account_id: &str,
        folder_id: &str,
        email_ids: Vec<&str>,
        flags: Vec<EmailFlag>,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait DeleteEmails {
    /// Mark the emails matching the given ids as deleted.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn delete_emails(
        &self,
        account_id: &str,
        folder_id: &str,
        email_ids: Vec<&str>,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait MoveEmails {
    /// Move the emails matching the given ids to another folder.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn move_emails(
        &self,
        account_id: &str,
        from_folder_id: &str,
        to_folder_id: &str,
        email_ids: Vec<&str>,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait CopyEmails {
    /// Copy the emails matching the given ids to another folder.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn copy_emails(
        &self,
        account_id: &str,
        from_folder_id: &str,
        to_folder_id: &str,
        email_ids: Vec<&str>,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait SendMessage {
    /// Send a raw RFC 822 message through the account's sending backend,
    /// saving a copy to its sent folder.
    ///
    /// # Errors
    ///
    /// Returns an error if the message cannot be sent.
    fn send_message(
        &self,
        account_id: &str,
        message: Vec<u8>,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait GetSenderAddress {
    /// The address outgoing mail of `account_id` is sent from.
    ///
    /// # Errors
    ///
    /// Returns an error if the account or its sender address cannot be
    /// resolved.
    fn get_sender_address(&self, account_id: &str) -> anyhow::Result<String>;
}

pub trait SaveDraft {
    /// Save a raw RFC 822 message as a draft in the account's drafts folder.
    ///
    /// # Errors
    ///
    /// Returns an error if the draft cannot be saved.
    fn save_draft(
        &self,
        account_id: &str,
        message: Vec<u8>,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}
