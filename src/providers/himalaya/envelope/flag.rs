use email::backend::feature::BackendFeatureSource;
use email::envelope::flag::Flags;

use super::{Flag, email_ids_to_usize};
use crate::api::email::EmailFlag;
use crate::api::email::commands::{AddEmailFlags, RemoveEmailFlags, SetEmailFlags};
use crate::providers::himalaya::HimalayaProvider;

fn to_flags(flags: Vec<EmailFlag>) -> Flags {
    flags.into_iter().map(Flag::from).collect()
}

impl AddEmailFlags for HimalayaProvider {
    async fn add_email_flags(
        &self,
        account_id: &str,
        folder_id: &str,
        email_ids: Vec<&str>,
        flags: Vec<EmailFlag>,
    ) -> anyhow::Result<()> {
        let ids = email_ids_to_usize(&email_ids)?;
        let flags = to_flags(flags);

        let backend = self
            .get_backend(account_id, |builder| {
                builder
                    .without_features()
                    .with_add_flags(BackendFeatureSource::Context)
            })
            .await?;

        backend
            .add_flags(folder_id, &ids, &flags)
            .await
            .map_err(|err| anyhow::anyhow!("failed to add flags: {err}"))
    }
}

impl RemoveEmailFlags for HimalayaProvider {
    async fn remove_email_flags(
        &self,
        account_id: &str,
        folder_id: &str,
        email_ids: Vec<&str>,
        flags: Vec<EmailFlag>,
    ) -> anyhow::Result<()> {
        let ids = email_ids_to_usize(&email_ids)?;
        let flags = to_flags(flags);

        let backend = self
            .get_backend(account_id, |builder| {
                builder
                    .without_features()
                    .with_remove_flags(BackendFeatureSource::Context)
            })
            .await?;

        backend
            .remove_flags(folder_id, &ids, &flags)
            .await
            .map_err(|err| anyhow::anyhow!("failed to remove flags: {err}"))
    }
}

impl SetEmailFlags for HimalayaProvider {
    async fn set_email_flags(
        &self,
        account_id: &str,
        folder_id: &str,
        email_ids: Vec<&str>,
        flags: Vec<EmailFlag>,
    ) -> anyhow::Result<()> {
        let ids = email_ids_to_usize(&email_ids)?;
        let flags = to_flags(flags);

        let backend = self
            .get_backend(account_id, |builder| {
                builder
                    .without_features()
                    .with_set_flags(BackendFeatureSource::Context)
            })
            .await?;

        backend
            .set_flags(folder_id, &ids, &flags)
            .await
            .map_err(|err| anyhow::anyhow!("failed to set flags: {err}"))
    }
}
