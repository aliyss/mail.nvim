use crate::api::account::Account;
use crate::api::account::commands::{GetAccount, ListAccounts};
use crate::providers::fake::{FakeProvider, accounts};

impl ListAccounts for FakeProvider {
    fn list_accounts(&self) -> anyhow::Result<Vec<Account>> {
        Ok(accounts())
    }
}

impl GetAccount for FakeProvider {
    fn get_account(&self, name: &str) -> anyhow::Result<Option<Account>> {
        Ok(accounts().into_iter().find(|account| account.name() == name))
    }

    fn get_default_account(&self) -> anyhow::Result<Account> {
        accounts()
            .into_iter()
            .find(Account::is_default)
            .ok_or_else(|| anyhow::anyhow!("no default account set"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_default_account() {
        let provider = FakeProvider;
        let account = provider
            .get_default_account()
            .expect("expected a default account");
        assert_eq!(account.name(), "nic@example.com");
    }

    #[test]
    fn get_unknown_account_returns_none() {
        let provider = FakeProvider;
        assert!(provider.get_account("nobody").expect("no error").is_none());
    }
}
