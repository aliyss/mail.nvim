use email::backend::feature::BackendFeatureSource;
use email::folder::purge::PurgeFolder;
use std::convert::Infallible;

use super::super::HimalayaProvider;
use crate::api::account::Account;
use crate::api::folder::commands::Purge;
use crate::providers::himalaya::account::himalaya_backend_from_account;

impl Purge for HimalayaProvider {
    type Error = Infallible;

    /// Purge all messages in the specified folder, regardless of their flags.
    /// This is a more aggressive operation than expunge.
    async fn folders_purge(
        &self,
        account: Option<&Account>,
        folder_id: &str,
    ) -> Result<(), Self::Error> {
        let backend = himalaya_backend_from_account(self, account, |builder| {
            builder
                .without_features()
                .with_purge_folder(BackendFeatureSource::Context)
        })
        .await?;

        backend
            .purge_folder(folder_id)
            .await
            .expect("failed to purge folder");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tokio;

    #[tokio::test]
    async fn folders_purge() {
        //     Test is commented out because purging folders will be handled after being able to
        //     create messages
    }
}
