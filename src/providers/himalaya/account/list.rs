use std::convert::Infallible;

use pimalaya_tui::himalaya::config::Accounts;

use super::super::HimalayaProvider;
use crate::api::account::Account;
use crate::api::account::commands::List;

impl List for HimalayaProvider {
    type Error = Infallible;

    fn accounts_list(&self) -> Result<Vec<Account>, Self::Error> {
        let accounts = Accounts::from(self.config.accounts.iter());

        Ok(accounts
            .iter()
            .map(|account| {
                Account::new(
                    account.name.clone(),
                    Some(account.backend.clone()),
                    account.default,
                )
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::config::Config;

    #[test]
    fn accounts_list() {
        let config = Config::builder()
            .build()
            .expect("Expected default builder to be valid");
        let himalaya_provider = HimalayaProvider::from_config(&config)
            .expect("Expected to create himalaya provider from default config");
        let accounts = himalaya_provider
            .accounts_list()
            .expect("Expected to list accounts");
        assert!(!accounts.is_empty(), "Expected at least one account");
    }
}
