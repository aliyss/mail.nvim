use anyhow::anyhow;
use email::backend::feature::BackendFeatureSource;
use email::folder::delete::DeleteFolder as _;

use crate::api::account::Account;
use crate::api::folder::commands::DeleteFolder;
use crate::providers::himalaya::HimalayaProvider;

impl DeleteFolder for HimalayaProvider {
    async fn delete_folder(&self, account: &Account, folder_id: &str) -> anyhow::Result<()> {
        self.get_backend(account, |builder| {
            builder
                .without_features()
                .with_delete_folder(BackendFeatureSource::Context)
        })
        .await?
        .delete_folder(folder_id)
        .await
        .map_err(|err| anyhow!(err))
    }
}

#[cfg(test)]
mod tests {
    use crate::api::account::commands::GetAccount;
    use crate::api::folder::commands::{CreateFolder as _, ListFolders as _};
    use crate::constants::{MAIL_APPLICATION, MAIL_ORGANIZATION};
    use tokio;

    use super::*;
    use crate::api::config::Config;

    #[tokio::test]
    async fn folders_delete() {
        let config = Config::builder()
            .build()
            .expect("expected default builder to be valid");
        let provider = HimalayaProvider::from_config(&config)
            .expect("expected to create himalaya provider from default config");
        let account = provider
            .get_default_account()
            .expect("failed to get default account");

        let folder_id = format!("{MAIL_ORGANIZATION}-{MAIL_APPLICATION}");

        provider
            .create_folder(&account, &folder_id)
            .await
            .expect("expected to create folders");
        let folders = provider
            .list_folders(&account)
            .await
            .expect("expected to list folders");
        assert!(
            folders.iter().any(|folder| folder.id() == folder_id),
            "expected to find created folder"
        );

        let () = provider
            .delete_folder(&account, &folder_id)
            .await
            .expect("expected to delete folders");
        let folders = provider
            .list_folders(&account)
            .await
            .expect("expected to list folders");

        assert!(
            folders.iter().all(|folder| folder.id() != folder_id),
            "expected to not find deleted folder"
        );
    }
}
