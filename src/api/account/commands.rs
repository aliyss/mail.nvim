use std::error;

use crate::api::account::Account;

pub trait List {
    type Error: error::Error + Send + Sync + 'static;

    /// Execute the list command using the provided mail provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn accounts_list(&self) -> Result<Vec<Account>, Self::Error>;
}

pub trait Get {
    type Error: error::Error + Send + Sync + 'static;

    /// Execute the get command using the provided mail provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn accounts_get(&self, name: Option<&str>) -> Result<Account, Self::Error>;
}
