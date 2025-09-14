pub mod arguments;
pub mod commands;
use std::collections::HashSet;

use chrono::{DateTime, Utc};

/// Represents the flag variants.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum EnvelopeFlag {
    Seen,
    Answered,
    Flagged,
    Deleted,
    Draft,
    Custom(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Mailbox {
    pub name: Option<String>,
    pub address: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    id: String,
    flags: HashSet<EnvelopeFlag>,
    subject: String,
    from: Mailbox,
    to: Mailbox,
    date: DateTime<Utc>,
    has_attachment: bool,
}

impl Envelope {
    #[must_use]
    pub fn new(
        id: String,
        flags: HashSet<EnvelopeFlag>,
        subject: String,
        from: Mailbox,
        to: Mailbox,
        date: DateTime<Utc>,
        has_attachment: bool,
    ) -> Self {
        Self {
            id,
            flags,
            subject,
            from,
            to,
            date,
            has_attachment,
        }
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn subject(&self) -> &str {
        &self.subject
    }

    #[must_use]
    pub fn from(&self) -> &Mailbox {
        &self.from
    }

    #[must_use]
    pub fn to(&self) -> &Mailbox {
        &self.to
    }

    #[must_use]
    pub fn date(&self) -> &DateTime<Utc> {
        &self.date
    }

    #[must_use]
    pub fn has_attachment(&self) -> bool {
        self.has_attachment
    }
}
