use pimalaya_tui::himalaya::config::{Account as HimalayaAccount, Accounts};

use crate::{
    api::account::{Account, List},
    providers::himalaya::HimalayaProvider,
};

/// List all existing accounts.
///
/// This command lists all the accounts defined in your TOML
/// configuration file.
#[derive(Debug)]
pub struct AccountListCommand {}

impl From<HimalayaAccount> for Account {
    fn from(account: HimalayaAccount) -> Self {
        Account {
            name: account.name,
            backends: Some(account.backend),
            default: account.default,
        }
    }
}

impl From<&HimalayaAccount> for Account {
    fn from(account: &HimalayaAccount) -> Self {
        Account {
            name: account.name.clone(),
            backends: Some(account.backend.clone()),
            default: account.default,
        }
    }
}

impl List<HimalayaProvider> for AccountListCommand {
    fn execute(self, provider: &HimalayaProvider) -> Result<Vec<Account>, String> {
        let accounts = Accounts::from(provider.config.accounts.iter());
        Ok(accounts.iter().map(Account::from).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::config::Config;

    #[test]
    fn list_accounts() {
        let config = Config::builder()
            .build()
            .expect("Expected default builder to be valid");
        let himalaya_provider = HimalayaProvider::try_from(config)
            .expect("Expected to create himalaya provider from default config");
        let command = AccountListCommand {};
        let accounts = command
            .execute(&himalaya_provider)
            .expect("Expected to list accounts");
        assert!(!accounts.is_empty(), "Expected at least one account");
    }
}
