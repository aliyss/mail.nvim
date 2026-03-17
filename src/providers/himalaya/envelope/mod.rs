use std::collections::HashSet;

use pimalaya_tui::himalaya::config::{
    Envelope as HimalayaEnvelope, Flag as HimalayaEnvelopeFlag, Mailbox as HimalayaMailbox,
};

use crate::api::email::{Email, EmailFlag, Mailbox};
use chrono::{DateTime, Utc};

mod list;

impl From<HimalayaEnvelopeFlag> for EmailFlag {
    fn from(flag: HimalayaEnvelopeFlag) -> Self {
        match flag {
            HimalayaEnvelopeFlag::Seen => EmailFlag::Seen,
            HimalayaEnvelopeFlag::Answered => EmailFlag::Answered,
            HimalayaEnvelopeFlag::Flagged => EmailFlag::Flagged,
            HimalayaEnvelopeFlag::Draft => EmailFlag::Draft,
            HimalayaEnvelopeFlag::Deleted => EmailFlag::Deleted,
            HimalayaEnvelopeFlag::Custom(custom) => EmailFlag::Custom(custom),
        }
    }
}

impl From<HimalayaMailbox> for Mailbox {
    fn from(mailbox: HimalayaMailbox) -> Self {
        Self {
            name: mailbox.name,
            address: mailbox.addr,
        }
    }
}

impl From<HimalayaEnvelope> for Email {
    fn from(envelope: HimalayaEnvelope) -> Self {
        let envelope_date_time = DateTime::parse_from_str(&envelope.date, "%Y-%m-%d %H:%M%z")
            .expect("Failed to parse date")
            .with_timezone(&Utc);

        let flags = HashSet::<HimalayaEnvelopeFlag>::clone(&envelope.flags)
            .into_iter()
            .map(EmailFlag::from)
            .collect::<HashSet<_>>();

        Email::new(
            envelope.id,
            flags,
            envelope.subject,
            envelope.from.into(),
            envelope.to.into(),
            envelope_date_time,
            envelope.has_attachment,
        )
    }
}
