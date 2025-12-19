mod get;
mod list;

use pimalaya_tui::himalaya::config::Account as HimalayaAccount;

use crate::api::account::Account;

impl From<HimalayaAccount> for Account {
    fn from(account: HimalayaAccount) -> Self {
        Account::new(account.name, Some(account.backend), account.default)
    }
}

impl From<Account> for HimalayaAccount {
    fn from(account: Account) -> Self {
        HimalayaAccount::new(
            account.name(),
            account.backend().unwrap_or_default(),
            account.is_default(),
        )
    }
}
