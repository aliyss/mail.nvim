use email::backend::feature::BackendFeatureSource;
use email::folder::expunge::ExpungeFolder;
use std::convert::Infallible;

use super::super::HimalayaProvider;
use crate::api::account::Account;
use crate::api::folder::commands::Expunge;
use crate::providers::himalaya::account::himalaya_backend_from_account;

impl Expunge for HimalayaProvider {
    type Error = Infallible;

    /// Purge (expunge) all messages marked as deleted in the specified folder.
    /// This is equivalent to the "EXPUNGE" command in IMAP.
    async fn folders_expunge(
        &self,
        account: Option<&Account>,
        folder_id: &str,
    ) -> Result<(), Self::Error> {
        let backend = himalaya_backend_from_account(self, account, |builder| {
            builder
                .without_features()
                .with_expunge_folder(BackendFeatureSource::Context)
        })
        .await?;

        backend
            .expunge_folder(folder_id)
            .await
            .expect("failed to expunge folder");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tokio;

    #[tokio::test]
    async fn folders_expunge() {
        //     Test is commented out because expunging folders will be handled after being able to
        //     create messages
    }
}
