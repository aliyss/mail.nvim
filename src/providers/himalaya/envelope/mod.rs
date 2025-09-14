use std::collections::HashSet;

use pimalaya_tui::himalaya::config::{
    Envelope as HimalayaEnvelope, Flag as HimalayaEnvelopeFlag, Mailbox as HimalayaMailbox,
};

use crate::api::envelope::{Envelope, EnvelopeFlag, Mailbox};
use chrono::{DateTime, Utc};

mod list;

impl From<HimalayaEnvelopeFlag> for EnvelopeFlag {
    fn from(flag: HimalayaEnvelopeFlag) -> Self {
        match flag {
            HimalayaEnvelopeFlag::Seen => EnvelopeFlag::Seen,
            HimalayaEnvelopeFlag::Answered => EnvelopeFlag::Answered,
            HimalayaEnvelopeFlag::Flagged => EnvelopeFlag::Flagged,
            HimalayaEnvelopeFlag::Draft => EnvelopeFlag::Draft,
            HimalayaEnvelopeFlag::Deleted => EnvelopeFlag::Deleted,
            HimalayaEnvelopeFlag::Custom(custom) => EnvelopeFlag::Custom(custom),
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

impl From<HimalayaEnvelope> for Envelope {
    fn from(envelope: HimalayaEnvelope) -> Self {
        let envelope_date_time = DateTime::parse_from_str(&envelope.date, "%Y-%m-%d %H:%M%z")
            .expect("Failed to parse date")
            .with_timezone(&Utc);

        let flags = HashSet::<HimalayaEnvelopeFlag>::clone(&envelope.flags)
            .into_iter()
            .map(EnvelopeFlag::from)
            .collect::<HashSet<_>>();

        Envelope::new(
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
