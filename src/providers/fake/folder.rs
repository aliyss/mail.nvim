use crate::api::folder::Folder;
use crate::api::folder::commands::{CreateFolder, DeleteFolder, GetFolder, ListFolders};
use crate::providers::fake::{FakeProvider, fake_delay, folders};

impl ListFolders for FakeProvider {
    async fn list_folders(&self, account_id: &str) -> anyhow::Result<Vec<Folder>> {
        fake_delay().await;
        Ok(folders(account_id))
    }
}

impl GetFolder for FakeProvider {
    async fn get_folder(
        &self,
        account_id: &str,
        folder_id: &str,
    ) -> anyhow::Result<Option<Folder>> {
        Ok(self
            .list_folders(account_id)
            .await?
            .into_iter()
            .find(|folder| folder.id() == folder_id))
    }
}

impl CreateFolder for FakeProvider {
    async fn create_folder(&self, _account_id: &str, _folder_name: &str) -> anyhow::Result<()> {
        fake_delay().await;
        Ok(())
    }
}

impl DeleteFolder for FakeProvider {
    async fn delete_folder(&self, _account_id: &str, _folder_id: &str) -> anyhow::Result<()> {
        fake_delay().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn list_folders_waits_the_fake_delay() {
        let provider = FakeProvider;
        let start = std::time::Instant::now();
        let folders = provider
            .list_folders("nic@example.com")
            .await
            .expect("expected folders");
        let elapsed = start.elapsed();

        assert!(
            elapsed >= crate::providers::fake::FAKE_DELAY,
            "expected the fake network delay, took {elapsed:?}"
        );
        assert!(folders.iter().any(|folder| folder.id() == "INBOX"));
    }

    #[tokio::test]
    async fn get_folder_filters_the_list() {
        let provider = FakeProvider;
        let folder = provider
            .get_folder("nic@example.com", "INBOX")
            .await
            .expect("expected the folder");
        assert_eq!(folder.expect("folder exists").id(), "INBOX");
    }

    #[tokio::test]
    async fn mutations_succeed() {
        let provider = FakeProvider;
        provider
            .create_folder("nic@example.com", "NewFolder")
            .await
            .expect("create should succeed");
        provider
            .delete_folder("nic@example.com", "NewFolder")
            .await
            .expect("delete should succeed");
    }
}
