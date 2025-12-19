use pimalaya_tui::himalaya::config::Accounts;

use crate::api::account::Account;
use crate::api::account::commands::ListAccounts;
use crate::providers::himalaya::HimalayaProvider;

impl ListAccounts for HimalayaProvider {
    fn list_accounts(&self) -> anyhow::Result<Vec<Account>> {
        Ok(Accounts::from(self.config.accounts.iter())
            .iter()
            .map(|a| Account::new(a.name.clone(), Some(a.backend.clone()), a.default))
            .collect())
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
            .expect("failed to build default config");
        let provider = config
            .to_provider()
            .expect("failed to create provider from config");

        let accounts = provider.list_accounts().expect("failed to list accounts");

        assert!(!accounts.is_empty(), "Expected at least one account");
    }
}
