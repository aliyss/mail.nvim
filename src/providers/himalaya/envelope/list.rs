use email::backend::feature::BackendFeatureSource;
use email::envelope::list::ListEnvelopesOptions as EnvelopeListEnvelopeOptions;

use crate::api::email::Email;
use crate::api::email::arguments::EmailListArguments;
use crate::api::email::commands::ListEmails;
use crate::providers::himalaya::HimalayaProvider;

impl ListEmails for HimalayaProvider {
    async fn list_emails(
        &self,
        account_id: &str,
        folder_id: Option<&str>,
        options: Option<EmailListArguments>,
    ) -> anyhow::Result<Vec<Email>> {
        let (himalaya_account_config, email_account_config) =
            self.get_account_config(account_id)?;

        let email_folder_id = match folder_id {
            Some(id) => id.to_owned(),
            None => email_account_config.get_inbox_folder_alias(),
        };

        let email_options = options.unwrap_or_default();
        let page = email_options.page_or_default();
        let per_page = email_options
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

        let list_email_options = EnvelopeListEnvelopeOptions {
            page,
            page_size: per_page,
            query: None,
        };

        let emails = backend
            .list_envelopes(&email_folder_id, list_email_options)
            .await
            .expect("failed to list emails");

        Ok(emails.iter().map(|email| email.clone().into()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::account::commands::GetAccount;
    use crate::api::config::Config;

    #[tokio::test]
    async fn emails_list() {
        let config = Config::builder()
            .build()
            .expect("expected default builder to be valid");
        let provider = HimalayaProvider::from_config(&config)
            .expect("expected to create himalaya provider from default config");
        let account = provider
            .get_default_account()
            .expect("failed to get default account");
        let emails = provider
            .list_emails(account.name(), None, None)
            .await
            .expect("expected to list emails");

        assert!(!emails.is_empty());
    }

    #[tokio::test]
    async fn emails_list_per_page() {
        let config = Config::builder()
            .build()
            .expect("expected default builder to be valid");
        let provider = HimalayaProvider::from_config(&config)
            .expect("expected to create himalaya provider from default config");
        let account = provider
            .get_default_account()
            .expect("failed to get default account");
        let options = EmailListArguments::new(
            Some(1), // page
            Some(1), // per_page
        );
        let emails = provider
            .list_emails(account.name(), None, Some(options))
            .await
            .expect("expected to list emails");

        assert!(emails.len() <= 1, "expected at most one email");
    }

    #[tokio::test]
    async fn emails_list_folder() {
        let config = Config::builder()
            .build()
            .expect("expected default builder to be valid");
        let provider = HimalayaProvider::from_config(&config)
            .expect("expected to create himalaya provider from default config");
        let account = provider
            .get_default_account()
            .expect("failed to get default account");
        let options = EmailListArguments::new(
            Some(1), // page
            Some(1), // per_page
        );
        let emails = provider
            .list_emails(account.name(), Some("Snoozed"), Some(options))
            .await
            .expect("expected to list emails");

        assert!(emails.len() <= 1, "expected at most one email");
    }
}
