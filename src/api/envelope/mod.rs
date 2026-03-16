pub mod arguments;
pub mod commands;
use std::collections::HashSet;

use chrono::{DateTime, Utc};

use crate::utils::render::table::render::{RenderTable, RowBuilder};

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

impl RenderTable for Vec<Envelope> {
    type Item = Envelope;

    fn headers(&self) -> Vec<String> {
        vec![
            "ID".to_string(),
            "Subject".to_string(),
            "From".to_string(),
            "To".to_string(),
            "Date".to_string(),
            "Has Attachment".to_string(),
            "Flags".to_string(),
        ]
    }

    fn rows(&self) -> Vec<RowBuilder> {
        self.iter()
            .map(|envelope| {
                RowBuilder::new()
                    .with_cell(envelope.id.clone())
                    .with_cell(envelope.subject.clone())
                    .with_cell(envelope.from.address.clone())
                    .with_cell(envelope.to.address.clone())
                    .with_cell(envelope.date.to_rfc3339())
                    .with_cell(if envelope.has_attachment {
                        "Yes".to_string()
                    } else {
                        "No".to_string()
                    })
                    .with_cell(
                        envelope
                            .flags
                            .iter()
                            .map(|flag| match flag {
                                EnvelopeFlag::Seen => "Seen".to_string(),
                                EnvelopeFlag::Answered => "Answered".to_string(),
                                EnvelopeFlag::Flagged => "Flagged".to_string(),
                                EnvelopeFlag::Deleted => "Deleted".to_string(),
                                EnvelopeFlag::Draft => "Draft".to_string(),
                                EnvelopeFlag::Custom(name) => name.clone(),
                            })
                            .collect::<Vec<String>>()
                            .join(", "),
                    )
            })
            .collect()
    }

    fn from_headers_and_rows(headers: Vec<String>, rows: Vec<RowBuilder>) -> Self {
        let mut envelopes: Vec<Envelope> = Vec::new();
        let id_index = headers
            .iter()
            .position(|h| h == "ID")
            .expect("Expected 'ID' header to be present in the table");
        let subject_index = headers.iter().position(|h| h == "Subject");
        let from_index = headers.iter().position(|h| h == "From");
        let to_index = headers.iter().position(|h| h == "To");
        let date_index = headers.iter().position(|h| h == "Date");
        let has_attachment_index = headers.iter().position(|h| h == "Has Attachment");
        let flags_index = headers.iter().position(|h| h == "Flags");

        for row in rows {
            let cells = row.cells;
            let id = match cells.get(id_index) {
                Some(cell) => cell.clone(),
                None => continue, // Skip rows without a name cell
            };

            let subject = subject_index.and_then(|index| cells.get(index).cloned());
            let from = from_index
                .and_then(|index| cells.get(index).cloned())
                .map_or_else(
                    || Mailbox {
                        name: None,
                        address: String::new(),
                    },
                    |cell| Mailbox {
                        name: None,
                        address: cell,
                    },
                );
            let to = to_index
                .and_then(|index| cells.get(index).cloned())
                .map_or_else(
                    || Mailbox {
                        name: None,
                        address: String::new(),
                    },
                    |cell| Mailbox {
                        name: None,
                        address: cell,
                    },
                );
            let date = date_index
                .and_then(|index| cells.get(index).cloned())
                .and_then(|cell| {
                    DateTime::parse_from_rfc3339(&cell)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok()
                });
            let has_attachment = has_attachment_index
                .and_then(|index| cells.get(index).cloned())
                .is_some_and(|cell| cell.to_lowercase() == "yes");
            let flags = flags_index.and_then(|index| {
                cells.get(index).map(|cell| {
                    cell.split(',')
                        .map(|flag_name| {
                            let trimmed_flag = flag_name.trim();
                            match trimmed_flag {
                                "Seen" => EnvelopeFlag::Seen,
                                "Answered" => EnvelopeFlag::Answered,
                                "Flagged" => EnvelopeFlag::Flagged,
                                "Deleted" => EnvelopeFlag::Deleted,
                                "Draft" => EnvelopeFlag::Draft,
                                custom => EnvelopeFlag::Custom(custom.to_string()),
                            }
                        })
                        .collect()
                })
            });

            envelopes.push(Envelope::new(
                id,
                flags.unwrap_or_default(),
                subject.unwrap_or_default(),
                from,
                to,
                date.unwrap_or_else(Utc::now),
                has_attachment,
            ));
        }
        envelopes
    }
}
