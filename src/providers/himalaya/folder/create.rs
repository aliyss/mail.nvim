use email::backend::feature::BackendFeatureSource;
use email::folder::add::AddFolder;
use pimalaya_tui::himalaya::backend::BackendBuilder;
use std::convert::Infallible;
use std::sync::Arc;

use super::super::HimalayaProvider;
use crate::api::account::Account;
use crate::api::folder::commands::Create;
use crate::providers::himalaya::account::himalaya_account_config_from_account;

impl Create for HimalayaProvider {
    type Error = Infallible;

    async fn folders_create(
        &self,
        account: Option<&Account>,
        folder_name: &str,
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
                    .with_add_folder(BackendFeatureSource::Context)
            },
        )
        .without_sending_backend()
        .build()
        .await
        .expect("failed to build backend");

        backend
            .add_folder(folder_name)
            .await
            .expect("failed to create folder");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use crate::{
    //     api::folder::commands::{Delete, List},
    //     constants::{MAIL_APPLICATION, MAIL_ORGANIZATION},
    // };
    use tokio;
    //
    // use super::*;
    // use crate::api::config::Config;
    //
    #[tokio::test]
    async fn folders_create() {
        //     Test is commented out because creating and deleting folders is handled in the delete test.
        //
        //
        //     let config = Config::builder()
        //         .build()
        //         .expect("expected default builder to be valid");
        //     let himalaya_provider = HimalayaProvider::from_config(&config)
        //         .expect("expected to create himalaya provider from default config");
        //
        //     let folder_id = format!("{MAIL_ORGANIZATION}-{MAIL_APPLICATION}");
        //
        //     let () = himalaya_provider
        //         .folders_create(None, &folder_id)
        //         .await
        //         .expect("expected to create folders");
        //     let folders = himalaya_provider
        //         .folders_list(None)
        //         .await
        //         .expect("expected to list folders");
        //     let () = himalaya_provider
        //         .folders_delete(None, &folder_id)
        //         .await
        //         .expect("expected to delete folders");
        //
        //     assert!(
        //         folders.iter().any(|f| f.id() == folder_id),
        //         "expected to find created folder"
        //     );
    }
}
