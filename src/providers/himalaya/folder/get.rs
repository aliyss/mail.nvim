use std::convert::Infallible;

use super::super::HimalayaProvider;
use crate::api::account::Account;
use crate::api::folder::Folder;
use crate::api::folder::commands::Get;
use crate::api::folder::commands::List;

impl Get for HimalayaProvider {
    type Error = Infallible;

    async fn folders_get(
        &self,
        account: Option<&Account>,
        folder_id: &str,
    ) -> Result<Folder, Self::Error> {
        let folders = self.folders_list(account).await?;

        let folder = folders
            .iter()
            .find(|folder| folder.id() == folder_id)
            .expect("folder not found");

        Ok(folder.clone())
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
        let folder = himalaya_provider
            .folders_get(None, folders[0].id())
            .await
            .expect("expected to get folder");
        assert_eq!(folder.id(), folders[0].id());
    }
}
