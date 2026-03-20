pub mod arguments;
pub mod commands;
use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};

use crate::{
    api::contact::Address,
    utils::render::{
        message::render::{InfoEntry, RenderMessage},
        table::render::{RenderTable, RowBuilder},
    },
};

/// Represents the flag variants.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum EmailFlag {
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
pub struct Email {
    id: String,
    flags: HashSet<EmailFlag>,
    subject: String,
    from: Mailbox,
    to: Mailbox,
    date: DateTime<Utc>,
    has_attachment: bool,
}

impl Email {
    #[must_use]
    pub fn new(
        id: String,
        flags: HashSet<EmailFlag>,
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

impl RenderTable for Vec<Email> {
    type Item = Email;

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
            .map(|email| {
                RowBuilder::new()
                    .with_cell(email.id.clone())
                    .with_cell(email.subject.clone())
                    .with_cell(email.from.address.clone())
                    .with_cell(email.to.address.clone())
                    .with_cell(email.date.to_rfc3339())
                    .with_cell(if email.has_attachment {
                        "Yes".to_string()
                    } else {
                        "No".to_string()
                    })
                    .with_cell(
                        email
                            .flags
                            .iter()
                            .map(|flag| match flag {
                                EmailFlag::Seen => "Seen".to_string(),
                                EmailFlag::Answered => "Answered".to_string(),
                                EmailFlag::Flagged => "Flagged".to_string(),
                                EmailFlag::Deleted => "Deleted".to_string(),
                                EmailFlag::Draft => "Draft".to_string(),
                                EmailFlag::Custom(name) => name.clone(),
                            })
                            .collect::<Vec<String>>()
                            .join(", "),
                    )
            })
            .collect()
    }

    fn from_headers_and_rows(headers: Vec<String>, rows: Vec<RowBuilder>) -> Self {
        let mut emails: Vec<Email> = Vec::new();
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
                                "Seen" => EmailFlag::Seen,
                                "Answered" => EmailFlag::Answered,
                                "Flagged" => EmailFlag::Flagged,
                                "Deleted" => EmailFlag::Deleted,
                                "Draft" => EmailFlag::Draft,
                                custom => EmailFlag::Custom(custom.to_string()),
                            }
                        })
                        .collect()
                })
            });

            emails.push(Email::new(
                id,
                flags.unwrap_or_default(),
                subject.unwrap_or_default(),
                from,
                to,
                date.unwrap_or_else(Utc::now),
                has_attachment,
            ));
        }
        emails
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailMessage {
    pub id: String,
    pub thread_id: Option<String>,
    pub subject: String,
    pub from: Vec<Address>,
    pub to: Vec<Address>,
    pub cc: Vec<Address>,
    pub bcc: Vec<Address>,
    pub date: Option<DateTime<Utc>>,
    pub body_text: String,
    pub body_html: Option<String>,
    pub attachment_ids: Vec<String>,
}

impl RenderTable for Vec<EmailMessage> {
    type Item = EmailMessage;

    fn headers(&self) -> Vec<String> {
        vec![
            "ID".to_string(),
            "Subject".to_string(),
            "From".to_string(),
            "To".to_string(),
            "CC".to_string(),
            "BCC".to_string(),
            "Date".to_string(),
            "Body Text".to_string(),
            "Body HTML".to_string(),
            "Attachment IDs".to_string(),
        ]
    }

    fn rows(&self) -> Vec<RowBuilder> {
        self.iter()
            .map(|email| {
                RowBuilder::new()
                    .with_cell(email.id.clone())
                    .with_cell(email.subject.clone())
                    .with_cell(
                        email
                            .from
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<String>>()
                            .join(", "),
                    )
                    .with_cell(
                        email
                            .to
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<String>>()
                            .join(", "),
                    )
                    .with_cell(
                        email
                            .cc
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<String>>()
                            .join(", "),
                    )
                    .with_cell(
                        email
                            .bcc
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<String>>()
                            .join(", "),
                    )
                    .with_cell(
                        email
                            .date
                            .map_or_else(|| "Unknown".to_string(), |d| d.to_rfc3339()),
                    )
                    .with_cell(email.body_text.clone())
                    .with_cell(
                        email
                            .body_html
                            .clone()
                            .unwrap_or_else(|| "None".to_string()),
                    )
                    .with_cell(email.attachment_ids.join(", "))
            })
            .collect()
    }

    fn from_headers_and_rows(headers: Vec<String>, rows: Vec<RowBuilder>) -> Self {
        let mut email_messages: Vec<EmailMessage> = Vec::new();
        let id_index = headers
            .iter()
            .position(|h| h == "ID")
            .expect("Expected 'ID' header to be present in the table");
        let subject_index = headers.iter().position(|h| h == "Subject");
        let from_index = headers.iter().position(|h| h == "From");
        let to_index = headers.iter().position(|h| h == "To");
        let cc_index = headers.iter().position(|h| h == "CC");
        let bcc_index = headers.iter().position(|h| h == "BCC");
        let date_index = headers.iter().position(|h| h == "Date");
        let body_text_index = headers.iter().position(|h| h == "Body Text");
        let body_html_index = headers.iter().position(|h| h == "Body HTML");
        let attachment_ids_index = headers.iter().position(|h| h == "Attachment IDs");

        for row in rows {
            let cells = row.cells;
            let id = match cells.get(id_index) {
                Some(cell) => cell.clone(),
                None => continue, // Skip rows without a name cell
            };

            let subject = subject_index
                .and_then(|index| cells.get(index).cloned())
                .unwrap_or_default();
            let from: Vec<Address> = from_index
                .and_then(|index| cells.get(index).cloned())
                .map_or_else(Vec::new, |cell| {
                    cell.split(',')
                        .map(|s| s.trim().to_string())
                        .filter_map(|s| s.parse().ok())
                        .collect()
                });
            let to: Vec<Address> = to_index
                .and_then(|index| cells.get(index).cloned())
                .map_or_else(Vec::new, |cell| {
                    cell.split(',')
                        .map(|s| s.trim().to_string())
                        .filter_map(|s| s.parse().ok())
                        .collect()
                });
            let cc: Vec<Address> = cc_index
                .and_then(|index| cells.get(index).cloned())
                .map_or_else(Vec::new, |cell| {
                    cell.split(',')
                        .map(|s| s.trim().to_string())
                        .filter_map(|s| s.parse().ok())
                        .collect()
                });
            let bcc: Vec<Address> = bcc_index
                .and_then(|index| cells.get(index).cloned())
                .map_or_else(Vec::new, |cell| {
                    cell.split(',')
                        .map(|s| s.trim().to_string())
                        .filter_map(|s| s.parse().ok())
                        .collect()
                });
            let date = date_index
                .and_then(|index| cells.get(index).cloned())
                .and_then(|cell| DateTime::parse_from_rfc3339(&cell).ok())
                .map(|dt| dt.with_timezone(&Utc));
            let body_text = body_text_index
                .and_then(|index| cells.get(index).cloned())
                .unwrap_or_default();
            let body_html = body_html_index
                .and_then(|index| cells.get(index).cloned())
                .filter(|s| s != "None");
            let attachment_ids = attachment_ids_index
                .and_then(|index| cells.get(index).cloned())
                .map_or_else(Vec::new, |cell| {
                    cell.split(',').map(|s| s.trim().to_string()).collect()
                });

            email_messages.push(EmailMessage {
                id,
                thread_id: None,
                subject,
                from,
                to,
                cc,
                bcc,
                date,
                body_text,
                body_html,
                attachment_ids,
            });
        }
        email_messages
    }
}

impl RenderMessage for EmailMessage {
    type Item = EmailMessage;

    fn info(&self) -> Vec<InfoEntry> {
        vec![
            InfoEntry {
                key: "ID".to_string(),
                value: self.id.clone(),
            },
            InfoEntry {
                key: "From".to_string(),
                value: self
                    .from
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", "),
            },
            InfoEntry {
                key: "To".to_string(),
                value: self
                    .to
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", "),
            },
            InfoEntry {
                key: "CC".to_string(),
                value: self
                    .cc
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", "),
            },
            InfoEntry {
                key: "BCC".to_string(),
                value: self
                    .bcc
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", "),
            },
            InfoEntry {
                key: "Subject".to_string(),
                value: self.subject.clone(),
            },
            InfoEntry {
                key: "Date".to_string(),
                value: self
                    .date
                    .map_or_else(|| "Unknown".to_string(), |d| d.to_rfc3339()),
            },
        ]
    }

    fn body(&self) -> String {
        self.body_text.clone()
    }

    fn from_data(info: HashMap<String, String>, body: String) -> Self {
        let id = info.get("ID").cloned().unwrap_or_default();
        let subject = info.get("Subject").cloned().unwrap_or_default();
        let from = info
            .get("From")
            .map(|s| {
                s.split(',')
                    .map(|s| s.trim().to_string())
                    .filter_map(|s| s.parse().ok())
                    .collect()
            })
            .unwrap_or_default();
        let to = info
            .get("To")
            .map(|s| {
                s.split(',')
                    .map(|s| s.trim().to_string())
                    .filter_map(|s| s.parse().ok())
                    .collect()
            })
            .unwrap_or_default();
        let cc = info
            .get("CC")
            .map(|s| {
                s.split(',')
                    .map(|s| s.trim().to_string())
                    .filter_map(|s| s.parse().ok())
                    .collect()
            })
            .unwrap_or_default();
        let bcc = info
            .get("BCC")
            .map(|s| {
                s.split(',')
                    .map(|s| s.trim().to_string())
                    .filter_map(|s| s.parse().ok())
                    .collect()
            })
            .unwrap_or_default();
        let date = info
            .get("Date")
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        EmailMessage {
            id,
            thread_id: None,
            subject,
            from,
            to,
            cc,
            bcc,
            date,
            body_text: body,
            body_html: None,
            attachment_ids: Vec::new(),
        }
    }
}
