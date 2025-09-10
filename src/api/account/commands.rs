use std::error;

use crate::api::account::Account;

pub trait List {
    type Error: error::Error + Send + Sync + 'static;

    /// Execute the list command using the provided mail provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn list(&self) -> Result<Vec<Account>, Self::Error>;
}
