use crate::api::account::Account;

pub trait ListAccounts {
    /// Execute the list command using the provided mail provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn list_accounts(&self) -> anyhow::Result<Vec<Account>>;
}

pub trait GetAccount {
    /// Get an account by name.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn get_account(&self, name: &str) -> anyhow::Result<Option<Account>>;

    // TODO: If `default_account` is not a standard thing, we can separate it into it's own trait
    // and make it optional.

    /// Get the default account for this mail provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the mail provider fails to get the default the default account.
    fn get_default_account(&self) -> anyhow::Result<Account>;
}
