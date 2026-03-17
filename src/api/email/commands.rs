use std::future::Future;

use crate::api::email::Email;
use crate::api::email::arguments::EmailListArguments;

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
