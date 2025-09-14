use email::backend::feature::BackendFeatureSource;
use email::envelope::list::ListEnvelopesOptions as EmailListEnvelopeOptions;
use std::convert::Infallible;

use super::super::HimalayaProvider;
use crate::api::account::Account;
use crate::api::envelope::Envelope;
use crate::api::envelope::arguments::EnvelopeListArguments;
use crate::api::envelope::commands::List;
use crate::providers::himalaya::account::{
    himalaya_account_config_from_account, himalaya_backend_from_account_config,
};

impl List for HimalayaProvider {
    type Error = Infallible;

    async fn envelopes_list(
        &self,
        account: Option<&Account>,
        folder_id: Option<&str>,
        options: Option<EnvelopeListArguments>,
    ) -> Result<Vec<Envelope>, Self::Error> {
        let (himalaya_account_config, email_account_config) =
            himalaya_account_config_from_account(self, account)?;

        let envelope_folder_id = match folder_id {
            Some(id) => id.to_string(),
            None => email_account_config.get_inbox_folder_alias(),
        };

        let envelope_options = options.unwrap_or_default();
        let page = envelope_options.page_or_default();
        let per_page = envelope_options
            .per_page_or_default(Some(|| email_account_config.get_envelope_list_page_size()));

        let backend = himalaya_backend_from_account_config(
            himalaya_account_config,
            email_account_config,
            |builder| {
                builder
                    .without_features()
                    .with_list_envelopes(BackendFeatureSource::Context)
            },
        )
        .await?;

        let list_envelope_options = EmailListEnvelopeOptions {
            page,
            page_size: per_page,
            query: None,
        };

        let envelopes = backend
            .list_envelopes(&envelope_folder_id, list_envelope_options)
            .await
            .expect("failed to list envelopes");

        Ok(envelopes
            .iter()
            .map(|envelope| envelope.clone().into())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use tokio;

    use super::*;
    use crate::api::config::Config;

    #[tokio::test]
    async fn envelopes_list() {
        let config = Config::builder()
            .build()
            .expect("expected default builder to be valid");
        let himalaya_provider = HimalayaProvider::from_config(&config)
            .expect("expected to create himalaya provider from default config");
        let envelopes = himalaya_provider
            .envelopes_list(None, None, None)
            .await
            .expect("expected to list envelopes");
        assert!(!envelopes.is_empty(), "expected at least one folder");
    }

    #[tokio::test]
    async fn envelopes_list_per_page() {
        let config = Config::builder()
            .build()
            .expect("expected default builder to be valid");
        let himalaya_provider = HimalayaProvider::from_config(&config)
            .expect("expected to create himalaya provider from default config");
        let options = EnvelopeListArguments::new(
            Some(1), // page
            Some(1), // per_page
        );
        let envelopes = himalaya_provider
            .envelopes_list(None, None, Some(options))
            .await
            .expect("expected to list envelopes");
        assert!(envelopes.len() <= 1, "expected at most one envelope");
    }
}
