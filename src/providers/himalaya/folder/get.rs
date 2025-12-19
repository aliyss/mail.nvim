use crate::api::account::Account;
use crate::api::folder::Folder;
use crate::api::folder::commands::{GetFolder, ListFolders as _};
use crate::providers::himalaya::HimalayaProvider;

impl GetFolder for HimalayaProvider {
    async fn get_folder(
        &self,
        account: &Account,
        folder_id: &str,
    ) -> anyhow::Result<Option<Folder>> {
        Ok(self
            .list_folders(account)
            .await?
            .into_iter()
            .find(|folder| folder.id() == folder_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::account::commands::GetAccount;
    use crate::api::config::Config;

    #[tokio::test]
    async fn folders_get() {
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
        let folder = provider
            .get_folder(&account, folders[0].id())
            .await
            .expect("expected to get folder");

        assert_ne!(folder, None);
    }
}
