use std::error;
use std::future::Future;

use crate::api::{
    account::Account,
    envelope::{Envelope, arguments::EnvelopeListArguments},
};

pub trait List {
    type Error: error::Error + Send + Sync + 'static;

    /// Execute the list command using the provided mail provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn envelopes_list(
        &self,
        account: Option<&Account>,
        folder_id: Option<&str>,
        options: Option<EnvelopeListArguments>,
    ) -> impl Future<Output = Result<Vec<Envelope>, Self::Error>> + Send;
}
