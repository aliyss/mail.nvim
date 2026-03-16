use anyhow::anyhow;
use email::backend::feature::BackendFeatureSource;
use email::folder::expunge::ExpungeFolder;

use super::super::HimalayaProvider;
use crate::api::folder::commands::ExpungeMessages;

impl ExpungeMessages for HimalayaProvider {
    /// Remove all messages marked as deleted in the specified folder.
    /// This is equivalent to the "EXPUNGE" command in IMAP.
    async fn expunge_messages(&self, account_id: &str, folder_id: &str) -> anyhow::Result<()> {
        self.get_backend(account_id, |builder| {
            builder
                .without_features()
                .with_expunge_folder(BackendFeatureSource::Context)
        })
        .await?
        .expunge_folder(folder_id)
        .await
        .map_err(|_| anyhow!("failed to expunge folder"))
    }
}

#[expect(unused_imports)]
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn folders_expunge() {
        //     Test is commented out because expunging folders will be handled after being able to
        //     create messages
    }
}
