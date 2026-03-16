use anyhow::anyhow;
use email::backend::feature::BackendFeatureSource;
use email::folder::purge::PurgeFolder;

use crate::api::folder::commands::PurgeMessages;
use crate::providers::himalaya::HimalayaProvider;

impl PurgeMessages for HimalayaProvider {
    /// Purge all messages in the specified folder, regardless of their flags.
    /// This is a more aggressive operation than expunge.
    async fn purge_messages(&self, account_id: &str, folder_id: &str) -> anyhow::Result<()> {
        self.get_backend(account_id, |builder| {
            builder
                .without_features()
                .with_purge_folder(BackendFeatureSource::Context)
        })
        .await?
        .purge_folder(folder_id)
        .await
        .map_err(|_| anyhow!("failed to purge folder"))
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
