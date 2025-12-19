use std::future::Future;

use crate::api::account::Account;
use crate::api::envelope::Envelope;
use crate::api::envelope::arguments::EnvelopeListArguments;

pub trait ListEnvelopes {
    /// Execute the list command using the provided mail provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    fn list_envelopes(
        &self,
        account: &Account,
        folder_id: Option<&str>,
        options: Option<EnvelopeListArguments>,
    ) -> impl Future<Output = anyhow::Result<Vec<Envelope>>> + Send;
}
