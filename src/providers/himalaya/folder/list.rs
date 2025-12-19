use email::backend::feature::BackendFeatureSource;
use email::folder::list::ListFolders as _;
use pimalaya_tui::himalaya::config::Folders as HimalayaFolders;

use super::super::HimalayaProvider;
use crate::api::account::Account;
use crate::api::folder::Folder;
use crate::api::folder::commands::ListFolders;

impl ListFolders for HimalayaProvider {
    async fn list_folders(&self, account: &Account) -> anyhow::Result<Vec<Folder>> {
        let backend = self
            .get_backend(account, |builder| {
                builder
                    .without_features()
                    .with_list_folders(BackendFeatureSource::Context)
            })
            .await?;

        let folders = HimalayaFolders::from(
            backend
                .list_folders()
                .await
                .expect("failed to list folders"),
        );

        Ok(folders.iter().map(|folder| folder.clone().into()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::account::commands::GetAccount as _;
    use crate::api::config::Config;

    #[tokio::test]
    async fn folders_list() {
        let config = Config::builder()
            .build()
            .expect("expected default builder to be valid");
        let provider = HimalayaProvider::from_config(&config)
            .expect("expected to create himalaya provider from default config");

        let account = provider
            .get_default_account()
            .expect("failed to get default account");
        let folders = provider
            .list_folders(&account)
            .await
            .expect("expected to list folders");

        assert!(!folders.is_empty());
    }
}
