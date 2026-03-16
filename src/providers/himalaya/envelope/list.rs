use email::backend::feature::BackendFeatureSource;
use email::envelope::list::ListEnvelopesOptions as EmailListEnvelopeOptions;

use crate::api::envelope::Envelope;
use crate::api::envelope::arguments::EnvelopeListArguments;
use crate::api::envelope::commands::ListEnvelopes;
use crate::providers::himalaya::HimalayaProvider;

impl ListEnvelopes for HimalayaProvider {
    async fn list_envelopes(
        &self,
        account_id: &str,
        folder_id: Option<&str>,
        options: Option<EnvelopeListArguments>,
    ) -> anyhow::Result<Vec<Envelope>> {
        let (himalaya_account_config, email_account_config) =
            self.get_account_config(account_id)?;

        let envelope_folder_id = match folder_id {
            Some(id) => id.to_owned(),
            None => email_account_config.get_inbox_folder_alias(),
        };

        let envelope_options = options.unwrap_or_default();
        let page = envelope_options.page_or_default();
        let per_page = envelope_options
            .per_page_or_default(Some(|| email_account_config.get_envelope_list_page_size()));

        let backend = Self::get_backend_from_config(
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
    use super::*;
    use crate::api::account::commands::GetAccount;
    use crate::api::config::Config;

    #[tokio::test]
    async fn envelopes_list() {
        let config = Config::builder()
            .build()
            .expect("expected default builder to be valid");
        let provider = HimalayaProvider::from_config(&config)
            .expect("expected to create himalaya provider from default config");
        let account = provider
            .get_default_account()
            .expect("failed to get default account");
        let envelopes = provider
            .list_envelopes(account.name(), None, None)
            .await
            .expect("expected to list envelopes");

        assert!(!envelopes.is_empty());
    }

    #[tokio::test]
    async fn envelopes_list_per_page() {
        let config = Config::builder()
            .build()
            .expect("expected default builder to be valid");
        let provider = HimalayaProvider::from_config(&config)
            .expect("expected to create himalaya provider from default config");
        let account = provider
            .get_default_account()
            .expect("failed to get default account");
        let options = EnvelopeListArguments::new(
            Some(1), // page
            Some(1), // per_page
        );
        let envelopes = provider
            .list_envelopes(account.name(), None, Some(options))
            .await
            .expect("expected to list envelopes");

        assert!(envelopes.len() <= 1, "expected at most one envelope");
    }

    #[tokio::test]
    async fn envelopes_list_folder() {
        let config = Config::builder()
            .build()
            .expect("expected default builder to be valid");
        let provider = HimalayaProvider::from_config(&config)
            .expect("expected to create himalaya provider from default config");
        let account = provider
            .get_default_account()
            .expect("failed to get default account");
        let options = EnvelopeListArguments::new(
            Some(1), // page
            Some(1), // per_page
        );
        let envelopes = provider
            .list_envelopes(account.name(), Some("Snoozed"), Some(options))
            .await
            .expect("expected to list envelopes");

        assert!(envelopes.len() <= 1, "expected at most one envelope");
    }
}
