use email::backend::feature::BackendFeatureSource;
use email::envelope::flag::{Flag, Flags};

use crate::api::email::commands::{GetSenderAddress, SaveDraft, SendMessage};
use crate::providers::himalaya::HimalayaProvider;

impl SendMessage for HimalayaProvider {
    async fn send_message(&self, account_id: &str, message: Vec<u8>) -> anyhow::Result<()> {
        let (himalaya_account_config, email_account_config) =
            self.get_account_config(account_id)?;

        // Sending needs both the SMTP/sendmail backend (to send) and the
        // `add_message` feature (to save a copy to the sent folder), so the
        // sending backend is kept this time (unlike reads and mutations).
        let backend = Self::get_backend_from_config(
            himalaya_account_config,
            email_account_config,
            |builder| {
                builder
                    .without_features()
                    .with_add_message(BackendFeatureSource::Context)
                    .with_send_message(BackendFeatureSource::Context)
            },
            true,
        )
        .await?;

        backend
            .send_message_then_save_copy(&message)
            .await
            .map_err(|err| anyhow::anyhow!("failed to send message: {err}"))?;

        Ok(())
    }
}

impl GetSenderAddress for HimalayaProvider {
    fn get_sender_address(&self, account_id: &str) -> anyhow::Result<String> {
        let (himalaya_config, _) = self.get_account_config(account_id)?;
        Ok(himalaya_config.email)
    }
}

impl SaveDraft for HimalayaProvider {
    async fn save_draft(&self, account_id: &str, message: Vec<u8>) -> anyhow::Result<()> {
        let (himalaya_account_config, email_account_config) =
            self.get_account_config(account_id)?;

        // Saving needs the `add_message` feature, but no sending backend.
        let backend = Self::get_backend_from_config(
            himalaya_account_config,
            email_account_config.clone(),
            |builder| {
                builder
                    .without_features()
                    .with_add_message(BackendFeatureSource::Context)
            },
            false,
        )
        .await?;

        let drafts = email_account_config.get_drafts_folder_alias();
        backend
            .add_message_with_flags(&drafts, &message, &Flags::from_iter([Flag::Draft]))
            .await
            .map_err(|err| anyhow::anyhow!("failed to save draft: {err}"))?;

        Ok(())
    }
}
