use email::backend::feature::BackendFeatureSource;

use crate::api::email::EmailMessage;
use crate::api::email::arguments::EmailListArguments;
use crate::api::email::commands::GetEmail;
use crate::providers::himalaya::HimalayaProvider;

impl GetEmail for HimalayaProvider {
    async fn get_emails(
        &self,
        account_id: &str,
        email_ids: Vec<&str>,
        folder_id: Option<&str>,
        _options: Option<EmailListArguments>,
    ) -> anyhow::Result<Vec<EmailMessage>> {
        let (himalaya_account_config, email_account_config) =
            self.get_account_config(account_id)?;

        let email_folder_id = match folder_id {
            Some(id) => id.to_owned(),
            None => email_account_config.get_inbox_folder_alias(),
        };

        let backend = Self::get_backend_from_config(
            himalaya_account_config,
            email_account_config,
            |builder| {
                builder
                    .without_features()
                    .with_get_messages(BackendFeatureSource::Context)
            },
        )
        .await?;

        let email_id_usizes = email_ids
            .iter()
            .map(|id| {
                id.parse::<usize>().map_err(|err| {
                    anyhow::anyhow!("failed to parse email id '{id}' as usize: {err}")
                })
            })
            .collect::<Result<Vec<usize>, _>>()?;

        let messages = backend
            .get_messages(&email_folder_id, &email_id_usizes)
            .await
            .expect("failed to get emails");

        Ok(messages
            .to_vec()
            .into_iter()
            .filter_map(|message| EmailMessage::try_from(message).ok())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::api::account::commands::GetAccount;
    // use crate::api::config::Config;

    #[tokio::test]
    async fn emails_get() {
        // TODO: implement test for get email command
        // let config = Config::builder()
        //     .build()
        //     .expect("expected default builder to be valid");
        // let provider = HimalayaProvider::from_config(&config)
        //     .expect("expected to create himalaya provider from default config");
        // let account = provider
        //     .get_default_account()
        //     .expect("failed to get default account");
        // let emails = provider
        //     .get_email(account.name(), None, None)
        //     .await
        //     .expect("expected to list emails");
        //
        // assert!(!emails.is_empty());
    }
}
