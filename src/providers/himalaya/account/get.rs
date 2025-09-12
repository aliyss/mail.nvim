use std::convert::Infallible;

use super::super::HimalayaProvider;
use crate::api::account::Account;
use crate::api::account::commands::{Get, List};

impl Get for HimalayaProvider {
    type Error = Infallible;

    /// Execute the get command using the provided mail provider.
    ////
    /// # Errors
    /// Returns an error if name is provided but the account is not found.
    /// Returns an error if name is not provided and no default account is set.
    fn accounts_get(&self, name: Option<&str>) -> Result<Account, Self::Error> {
        let accounts = self.accounts_list().expect("Failed to list accounts");

        let account = match name {
            Some(name) => accounts
                .iter()
                .find(|account| account.name() == name)
                .expect("Account not found"),
            None => accounts
                .iter()
                .find(|account| account.is_default())
                .expect("No default account set"),
        };

        Ok(account.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::config::Config;

    #[test]
    fn accounts_get() {
        let config = Config::builder()
            .build()
            .expect("Expected default builder to be valid");
        let himalaya_provider = HimalayaProvider::from_config(&config)
            .expect("Expected to create himalaya provider from default config");
        let accounts = himalaya_provider
            .accounts_get(None)
            .expect("Expected to get account");
        assert!(
            !accounts.name().is_empty(),
            "Expected account to have a name"
        );
    }
}
