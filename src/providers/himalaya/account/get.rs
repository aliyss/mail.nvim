use anyhow::anyhow;

use crate::api::account::Account;
use crate::api::account::commands::{GetAccount, ListAccounts};
use crate::providers::himalaya::HimalayaProvider;

impl GetAccount for HimalayaProvider {
    fn get_account(&self, name: &str) -> anyhow::Result<Option<Account>> {
        let account = self
            .list_accounts()?
            .into_iter()
            .find(|account| account.name() == name);

        Ok(account)
    }

    fn get_default_account(&self) -> anyhow::Result<Account> {
        self.list_accounts()?
            .into_iter()
            .find(Account::is_default)
            .ok_or(anyhow!("no default account set"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::config::Config;

    #[test]
    fn get_account() {
        // let config = Config::builder()
        //     .build()
        //     .expect("expected default configuration to be valid");
        // let provider = HimalayaProvider::from_config(&config)
        //     .expect("failed to create Himalaya provider using default config");
        // let account = provider
        //     .get_account("mock_account")
        //     .expect("failed to get accounts");

        // assert_ne!(account, None, "failed to get account");
    }

    #[test]
    fn get_default_account() {
        let config = Config::builder()
            .build()
            .expect("expected default configuration to be valid");
        let provider = HimalayaProvider::from_config(&config)
            .expect("failed to create Himalaya provider using default config");

        _ = provider
            .get_default_account()
            .expect("failed to get default account");
    }
}
