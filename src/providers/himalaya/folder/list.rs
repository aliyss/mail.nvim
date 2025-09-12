use email::{backend::feature::BackendFeatureSource, folder::list::ListFolders};
use pimalaya_tui::himalaya::backend::BackendBuilder;
use pimalaya_tui::himalaya::config::Folders as HimalayaFolders;
use std::convert::Infallible;
use std::sync::Arc;

use super::super::HimalayaProvider;
use crate::api::account::Account;
use crate::api::folder::Folder;
use crate::api::folder::commands::List;
use crate::providers::himalaya::account::himalaya_account_config_from_account;

impl List for HimalayaProvider {
    type Error = Infallible;

    async fn folders_list(&self, account: Option<&Account>) -> Result<Vec<Folder>, Self::Error> {
        let (himalaya_account_config, email_account_config) =
            himalaya_account_config_from_account(self, account)
                .expect("failed to get account config");
        // TODO: Cache backend / add it to himalaya provider to avoid rebuilding it every time?
        // Account selection may change, so we need to rebuild the backend every time for now.
        // Currently this is also how himalaya CLI works.
        let backend = BackendBuilder::new(
            himalaya_account_config.into(),
            Arc::new(email_account_config),
            |builder| {
                builder
                    .without_features()
                    .with_list_folders(BackendFeatureSource::Context)
            },
        )
        .without_sending_backend()
        .build()
        .await
        .expect("failed to build backend");

        let folders = HimalayaFolders::from(
            backend
                .list_folders()
                .await
                .expect("failed to list folders"),
        );

        Ok(folders.iter().map(std::convert::Into::into).collect())
    }
}

#[cfg(test)]
mod tests {
    use tokio;

    use super::*;
    use crate::api::config::Config;

    #[tokio::test]
    async fn accounts_list() {
        let config = Config::builder()
            .build()
            .expect("expected default builder to be valid");
        let himalaya_provider = HimalayaProvider::from_config(&config)
            .expect("expected to create himalaya provider from default config");
        let folders = himalaya_provider
            .folders_list(None)
            .await
            .expect("expected to list folders");
        assert!(!folders.is_empty(), "expected at least one folder");
    }
}
