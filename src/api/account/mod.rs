pub mod commands;

use pimalaya_tui::himalaya::config::Account as HimalayaAccount;

pub struct Account {
    name: String,
    backend: Option<String>,
    default: bool,
}

impl Account {
    #[must_use]
    pub fn new(name: String, backend: Option<String>, default: bool) -> Self {
        Self {
            name,
            backend,
            default,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn backends(&self) -> Option<&str> {
        self.backend.as_deref()
    }

    #[must_use]
    pub fn is_default(&self) -> bool {
        self.default
    }
}

impl From<HimalayaAccount> for Account {
    fn from(account: HimalayaAccount) -> Self {
        Account::new(account.name, Some(account.backend), account.default)
    }
}
