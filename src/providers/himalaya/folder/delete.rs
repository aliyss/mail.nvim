use email::backend::feature::BackendFeatureSource;
use email::folder::delete::DeleteFolder;
use pimalaya_tui::himalaya::backend::BackendBuilder;
use std::convert::Infallible;
use std::sync::Arc;

use super::super::HimalayaProvider;
use crate::api::account::Account;
use crate::api::folder::commands::Delete;
use crate::providers::himalaya::account::himalaya_account_config_from_account;

impl Delete for HimalayaProvider {
    type Error = Infallible;

    async fn folders_delete(
        &self,
        account: Option<&Account>,
        folder_id: &str,
    ) -> Result<(), Self::Error> {
        let (himalaya_account_config, email_account_config) =
            himalaya_account_config_from_account(self, account)
                .expect("failed to get account config");
        let backend = BackendBuilder::new(
            himalaya_account_config.into(),
            Arc::new(email_account_config),
            |builder| {
                builder
                    .without_features()
                    .with_delete_folder(BackendFeatureSource::Context)
            },
        )
        .without_sending_backend()
        .build()
        .await
        .expect("failed to build backend");

        backend
            .delete_folder(folder_id)
            .await
            .expect("failed to delete folder");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        api::folder::commands::{Create, List},
        constants::{MAIL_APPLICATION, MAIL_ORGANIZATION},
    };
    use tokio;

    use super::*;
    use crate::api::config::Config;

    #[tokio::test]
    async fn folders_delete() {
        let config = Config::builder()
            .build()
            .expect("expected default builder to be valid");
        let himalaya_provider = HimalayaProvider::from_config(&config)
            .expect("expected to create himalaya provider from default config");

        let folder_id = format!("{MAIL_ORGANIZATION}-{MAIL_APPLICATION}");

        let () = himalaya_provider
            .folders_create(None, &folder_id)
            .await
            .expect("expected to create folders");
        let folders = himalaya_provider
            .folders_list(None)
            .await
            .expect("expected to list folders");
        assert!(
            folders.iter().any(|folder| folder.id() == folder_id),
            "expected to find created folder"
        );

        let () = himalaya_provider
            .folders_delete(None, &folder_id)
            .await
            .expect("expected to delete folders");
        let folders = himalaya_provider
            .folders_list(None)
            .await
            .expect("expected to list folders");

        assert!(
            folders.iter().all(|folder| folder.id() != folder_id),
            "expected to not find deleted folder"
        );
    }
}
