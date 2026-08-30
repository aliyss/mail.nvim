use crate::api::account::Account;
use crate::api::account::commands::{GetAccount, ListAccounts};
use crate::api::email::EmailMessage;
use crate::api::email::arguments::EmailListArguments;
use crate::api::email::commands::{
    AddEmailFlags, CopyEmails, DeleteEmails, GetEmail, GetSenderAddress, ListEmails, ListThreads,
    MoveEmails, RemoveEmailFlags, SaveDraft, SendMessage, SetEmailFlags,
};
use crate::api::email::{Email, EmailFlag, ThreadedEmail};
use crate::api::folder::Folder;
use crate::api::folder::commands::{CreateFolder, DeleteFolder, GetFolder, ListFolders};

pub mod fake;
pub mod himalaya;

pub use fake::FakeProvider;
pub use himalaya::HimalayaProvider;

pub trait Provider:
    GetAccount
    + GetFolder
    + ListAccounts
    + ListFolders
    + DeleteFolder
    + CreateFolder
    + ListEmails
    + GetEmail
    + ListThreads
    + AddEmailFlags
    + RemoveEmailFlags
    + SetEmailFlags
    + DeleteEmails
    + MoveEmails
    + CopyEmails
    + SendMessage
    + SaveDraft
    + GetSenderAddress
    + Clone
{
}
impl<
    T: GetAccount
        + GetFolder
        + ListAccounts
        + ListFolders
        + DeleteFolder
        + CreateFolder
        + ListEmails
        + GetEmail
        + ListThreads
        + AddEmailFlags
        + RemoveEmailFlags
        + SetEmailFlags
        + DeleteEmails
        + MoveEmails
        + CopyEmails
        + SendMessage
        + SaveDraft
        + GetSenderAddress
        + Clone,
> Provider for T
{
}

/// The concrete provider selected by the configuration, so
/// [`Config::to_provider`](crate::api::config::Config::to_provider) can return
/// a single type no matter which backend is configured.
#[derive(Debug, Clone)]
pub enum AnyProvider {
    /// The real Himalaya-backed provider.
    Himalaya(HimalayaProvider),
    /// The in-memory provider used by the tests.
    Fake(FakeProvider),
}

impl ListAccounts for AnyProvider {
    fn list_accounts(&self) -> anyhow::Result<Vec<Account>> {
        match self {
            Self::Himalaya(provider) => provider.list_accounts(),
            Self::Fake(provider) => provider.list_accounts(),
        }
    }
}

impl GetAccount for AnyProvider {
    fn get_account(&self, name: &str) -> anyhow::Result<Option<Account>> {
        match self {
            Self::Himalaya(provider) => provider.get_account(name),
            Self::Fake(provider) => provider.get_account(name),
        }
    }

    fn get_default_account(&self) -> anyhow::Result<Account> {
        match self {
            Self::Himalaya(provider) => provider.get_default_account(),
            Self::Fake(provider) => provider.get_default_account(),
        }
    }
}

impl ListFolders for AnyProvider {
    async fn list_folders(&self, account_id: &str) -> anyhow::Result<Vec<Folder>> {
        match self {
            Self::Himalaya(provider) => provider.list_folders(account_id).await,
            Self::Fake(provider) => provider.list_folders(account_id).await,
        }
    }
}

impl GetFolder for AnyProvider {
    async fn get_folder(
        &self,
        account_id: &str,
        folder_id: &str,
    ) -> anyhow::Result<Option<Folder>> {
        match self {
            Self::Himalaya(provider) => provider.get_folder(account_id, folder_id).await,
            Self::Fake(provider) => provider.get_folder(account_id, folder_id).await,
        }
    }
}

impl CreateFolder for AnyProvider {
    async fn create_folder(&self, account_id: &str, folder_name: &str) -> anyhow::Result<()> {
        match self {
            Self::Himalaya(provider) => provider.create_folder(account_id, folder_name).await,
            Self::Fake(provider) => provider.create_folder(account_id, folder_name).await,
        }
    }
}

impl DeleteFolder for AnyProvider {
    async fn delete_folder(&self, account_id: &str, folder_id: &str) -> anyhow::Result<()> {
        match self {
            Self::Himalaya(provider) => provider.delete_folder(account_id, folder_id).await,
            Self::Fake(provider) => provider.delete_folder(account_id, folder_id).await,
        }
    }
}

impl ListEmails for AnyProvider {
    async fn list_emails(
        &self,
        account_id: &str,
        folder_id: Option<&str>,
        options: Option<EmailListArguments>,
    ) -> anyhow::Result<Vec<Email>> {
        match self {
            Self::Himalaya(provider) => provider.list_emails(account_id, folder_id, options).await,
            Self::Fake(provider) => provider.list_emails(account_id, folder_id, options).await,
        }
    }
}

impl GetEmail for AnyProvider {
    async fn get_emails(
        &self,
        account_id: &str,
        email_ids: Vec<&str>,
        folder_id: Option<&str>,
        options: Option<EmailListArguments>,
    ) -> anyhow::Result<Vec<EmailMessage>> {
        match self {
            Self::Himalaya(provider) => {
                provider.get_emails(account_id, email_ids, folder_id, options).await
            }
            Self::Fake(provider) => {
                provider.get_emails(account_id, email_ids, folder_id, options).await
            }
        }
    }
}

impl ListThreads for AnyProvider {
    async fn list_threads(
        &self,
        account_id: &str,
        email_id: &str,
        folder_id: Option<&str>,
    ) -> anyhow::Result<Vec<ThreadedEmail>> {
        match self {
            Self::Himalaya(provider) => provider.list_threads(account_id, email_id, folder_id).await,
            Self::Fake(provider) => provider.list_threads(account_id, email_id, folder_id).await,
        }
    }
}

impl AddEmailFlags for AnyProvider {
    async fn add_email_flags(
        &self,
        account_id: &str,
        folder_id: &str,
        email_ids: Vec<&str>,
        flags: Vec<EmailFlag>,
    ) -> anyhow::Result<()> {
        match self {
            Self::Himalaya(provider) => {
                provider.add_email_flags(account_id, folder_id, email_ids, flags).await
            }
            Self::Fake(provider) => {
                provider.add_email_flags(account_id, folder_id, email_ids, flags).await
            }
        }
    }
}

impl RemoveEmailFlags for AnyProvider {
    async fn remove_email_flags(
        &self,
        account_id: &str,
        folder_id: &str,
        email_ids: Vec<&str>,
        flags: Vec<EmailFlag>,
    ) -> anyhow::Result<()> {
        match self {
            Self::Himalaya(provider) => {
                provider.remove_email_flags(account_id, folder_id, email_ids, flags).await
            }
            Self::Fake(provider) => {
                provider.remove_email_flags(account_id, folder_id, email_ids, flags).await
            }
        }
    }
}

impl SetEmailFlags for AnyProvider {
    async fn set_email_flags(
        &self,
        account_id: &str,
        folder_id: &str,
        email_ids: Vec<&str>,
        flags: Vec<EmailFlag>,
    ) -> anyhow::Result<()> {
        match self {
            Self::Himalaya(provider) => {
                provider.set_email_flags(account_id, folder_id, email_ids, flags).await
            }
            Self::Fake(provider) => {
                provider.set_email_flags(account_id, folder_id, email_ids, flags).await
            }
        }
    }
}

impl DeleteEmails for AnyProvider {
    async fn delete_emails(
        &self,
        account_id: &str,
        folder_id: &str,
        email_ids: Vec<&str>,
    ) -> anyhow::Result<()> {
        match self {
            Self::Himalaya(provider) => provider.delete_emails(account_id, folder_id, email_ids).await,
            Self::Fake(provider) => provider.delete_emails(account_id, folder_id, email_ids).await,
        }
    }
}

impl MoveEmails for AnyProvider {
    async fn move_emails(
        &self,
        account_id: &str,
        from_folder_id: &str,
        to_folder_id: &str,
        email_ids: Vec<&str>,
    ) -> anyhow::Result<()> {
        match self {
            Self::Himalaya(provider) => {
                provider.move_emails(account_id, from_folder_id, to_folder_id, email_ids).await
            }
            Self::Fake(provider) => {
                provider.move_emails(account_id, from_folder_id, to_folder_id, email_ids).await
            }
        }
    }
}

impl CopyEmails for AnyProvider {
    async fn copy_emails(
        &self,
        account_id: &str,
        from_folder_id: &str,
        to_folder_id: &str,
        email_ids: Vec<&str>,
    ) -> anyhow::Result<()> {
        match self {
            Self::Himalaya(provider) => {
                provider.copy_emails(account_id, from_folder_id, to_folder_id, email_ids).await
            }
            Self::Fake(provider) => {
                provider.copy_emails(account_id, from_folder_id, to_folder_id, email_ids).await
            }
        }
    }
}

impl SendMessage for AnyProvider {
    async fn send_message(&self, account_id: &str, message: Vec<u8>) -> anyhow::Result<()> {
        match self {
            Self::Himalaya(provider) => provider.send_message(account_id, message).await,
            Self::Fake(provider) => provider.send_message(account_id, message).await,
        }
    }
}

impl GetSenderAddress for AnyProvider {
    fn get_sender_address(&self, account_id: &str) -> anyhow::Result<String> {
        match self {
            Self::Himalaya(provider) => provider.get_sender_address(account_id),
            Self::Fake(provider) => provider.get_sender_address(account_id),
        }
    }
}

impl SaveDraft for AnyProvider {
    async fn save_draft(&self, account_id: &str, message: Vec<u8>) -> anyhow::Result<()> {
        match self {
            Self::Himalaya(provider) => provider.save_draft(account_id, message).await,
            Self::Fake(provider) => provider.save_draft(account_id, message).await,
        }
    }
}
