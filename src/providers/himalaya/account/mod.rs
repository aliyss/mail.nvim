use crate::api::account::commands::Get;
use pimalaya_tui::{himalaya::config::HimalayaTomlAccountConfig, terminal::config::TomlConfig};
use std::convert::Infallible;

use email::{account::config::AccountConfig, config::Config as EmailConfig};
use pimalaya_tui::himalaya::config::Account as HimalayaAccount;

use crate::{api::account::Account, providers::himalaya::HimalayaProvider};

mod get;
mod list;

impl From<HimalayaAccount> for Account {
    fn from(account: HimalayaAccount) -> Self {
        Account::new(account.name, Some(account.backend), account.default)
    }
}

impl From<Account> for HimalayaAccount {
    fn from(account: Account) -> Self {
        HimalayaAccount::new(
            account.name(),
            account.backends().unwrap_or_default(),
            account.is_default(),
        )
    }
}

// TODO: Move this to himalaya provider?
pub fn himalaya_account_config_from_account(
    himalaya_provider: &HimalayaProvider,
    account: Option<&Account>,
) -> Result<(HimalayaTomlAccountConfig, AccountConfig), Infallible> {
    let current_account = match account {
        Some(acc) => acc,
        None => &himalaya_provider.accounts_get(None)?,
    };

    let (himalaya_account_config_name, himalaya_account_config) = himalaya_provider
        .config()
        .get_account_config(current_account.name())
        .expect("failed to get account config");

    let email_account_config = Into::<EmailConfig>::into(himalaya_provider.config().clone())
        .account(himalaya_account_config_name)
        .expect("failed to get email account config");

    Ok((himalaya_account_config, email_account_config))
}
