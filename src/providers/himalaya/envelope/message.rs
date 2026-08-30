use email::backend::feature::BackendFeatureSource;

use super::email_ids_to_usize;
use crate::api::email::commands::{CopyEmails, DeleteEmails, MoveEmails};
use crate::providers::himalaya::HimalayaProvider;

impl DeleteEmails for HimalayaProvider {
    async fn delete_emails(
        &self,
        account_id: &str,
        folder_id: &str,
        email_ids: Vec<&str>,
    ) -> anyhow::Result<()> {
        let ids = email_ids_to_usize(&email_ids)?;

        let backend = self
            .get_backend(account_id, |builder| {
                builder
                    .without_features()
                    .with_delete_messages(BackendFeatureSource::Context)
            })
            .await?;

        backend
            .delete_messages(folder_id, &ids)
            .await
            .map_err(|err| anyhow::anyhow!("failed to delete emails: {err}"))
    }
}

impl MoveEmails for HimalayaProvider {
    async fn move_emails(
        &self,
        account_id: &str,
        from_folder_id: &str,
        to_folder_id: &str,
        email_ids: Vec<&str>,
    ) -> anyhow::Result<()> {
        let ids = email_ids_to_usize(&email_ids)?;

        let backend = self
            .get_backend(account_id, |builder| {
                builder
                    .without_features()
                    .with_move_messages(BackendFeatureSource::Context)
            })
            .await?;

        backend
            .move_messages(from_folder_id, to_folder_id, &ids)
            .await
            .map_err(|err| anyhow::anyhow!("failed to move emails: {err}"))
    }
}

impl CopyEmails for HimalayaProvider {
    async fn copy_emails(
        &self,
        account_id: &str,
        from_folder_id: &str,
        to_folder_id: &str,
        email_ids: Vec<&str>,
    ) -> anyhow::Result<()> {
        let ids = email_ids_to_usize(&email_ids)?;

        let backend = self
            .get_backend(account_id, |builder| {
                builder
                    .without_features()
                    .with_copy_messages(BackendFeatureSource::Context)
            })
            .await?;

        backend
            .copy_messages(from_folder_id, to_folder_id, &ids)
            .await
            .map_err(|err| anyhow::anyhow!("failed to copy emails: {err}"))
    }
}
