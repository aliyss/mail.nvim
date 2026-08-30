pub mod arguments;
pub mod commands;
use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};

use crate::{
    api::contact::Address,
    utils::render::{
        message::render::{InfoEntry, RenderMessage},
        table::marked::HasId,
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
    pub fn flags(&self) -> &HashSet<EmailFlag> {
        &self.flags
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

impl HasId for Email {
    fn id(&self) -> &str {
        self.id.as_str()
    }
}

/// The highlight group of an email's subject cell, coding its state:
/// deleted emails are dimmed, flagged ones yellow, answered ones green and
/// unread ones bold. The first matching state wins.
fn subject_style(email: &Email) -> Option<&'static str> {
    if email.flags.contains(&EmailFlag::Deleted) {
        Some("MailTableDeleted")
    } else if email.flags.contains(&EmailFlag::Flagged) {
        Some("MailTableFlagged")
    } else if email.flags.contains(&EmailFlag::Answered) {
        Some("MailTableAnswered")
    } else if !email.flags.contains(&EmailFlag::Seen) {
        Some("MailTableUnread")
    } else {
        None
    }
}

/// The highlight group of an email's attachment cell: emails with attached
/// files stand out, plain ones stay unstyled.
fn attachment_style(email: &Email) -> Option<&'static str> {
    email.has_attachment.then_some("MailTableAttachment")
}

impl HasId for ThreadedEmail {
    fn id(&self) -> &str {
        self.email.id()
    }
}

#[cfg(test)]
mod subject_style_tests {
    use super::*;

    fn email_with(flags: &[EmailFlag]) -> Email {
        Email::new(
            "1".into(),
            flags.iter().cloned().collect(),
            "Subject".into(),
            Mailbox::default(),
            Mailbox::default(),
            Utc::now(),
            false,
        )
    }

    fn attached_email() -> Email {
        Email::new(
            "1".into(),
            HashSet::new(),
            "Subject".into(),
            Mailbox::default(),
            Mailbox::default(),
            Utc::now(),
            true,
        )
    }

    #[test]
    fn unread_emails_are_bold() {
        assert_eq!(subject_style(&email_with(&[])), Some("MailTableUnread"));
    }

    #[test]
    fn read_emails_have_no_style() {
        assert_eq!(subject_style(&email_with(&[EmailFlag::Seen])), None);
    }

    #[test]
    fn flagged_wins_over_unread() {
        let email = email_with(&[EmailFlag::Flagged]);
        assert_eq!(subject_style(&email), Some("MailTableFlagged"));
    }

    #[test]
    fn answered_emails_are_green() {
        let email = email_with(&[EmailFlag::Answered]);
        assert_eq!(subject_style(&email), Some("MailTableAnswered"));
    }

    #[test]
    fn answered_wins_over_unread_but_not_over_flagged() {
        let answered_unread = email_with(&[EmailFlag::Answered]);
        assert_eq!(subject_style(&answered_unread), Some("MailTableAnswered"));

        let flagged_answered = email_with(&[EmailFlag::Flagged, EmailFlag::Answered]);
        assert_eq!(subject_style(&flagged_answered), Some("MailTableFlagged"));
    }

    #[test]
    fn deleted_wins_over_everything() {
        let email = email_with(&[EmailFlag::Deleted, EmailFlag::Flagged]);
        assert_eq!(subject_style(&email), Some("MailTableDeleted"));
    }

    #[test]
    fn attachments_color_the_attachment_cell() {
        assert_eq!(attachment_style(&email_with(&[])), None);
        assert_eq!(
            attachment_style(&attached_email()),
            Some("MailTableAttachment")
        );
    }

    #[test]
    fn attachment_rows_carry_the_cell_style() {
        let rows = vec![attached_email()].rows();
        // Columns: ID, Subject, From, To, Date, Has Attachment, Flags.
        assert_eq!(rows[0].styles[5], Some("MailTableAttachment"));
    }

    #[test]
    fn styled_rows_carry_the_subject_style() {
        let emails = vec![email_with(&[])];
        let rows = emails.rows();
        assert_eq!(rows[0].styles[0], None); // id column
        assert_eq!(rows[0].styles[1], Some("MailTableUnread")); // subject
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
                    .with_cell_styled(email.subject.clone(), subject_style(email))
                    .with_cell(email.from.address.clone())
                    .with_cell(email.to.address.clone())
                    .with_cell(email.date.to_rfc3339())
                    .with_cell_styled(
                        if email.has_attachment {
                            "Yes".to_string()
                        } else {
                            "No".to_string()
                        },
                        attachment_style(email),
                    )
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

/// An email together with its indentation level within its thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadedEmail {
    depth: usize,
    email: Email,
}

impl ThreadedEmail {
    #[must_use]
    pub fn new(depth: usize, email: Email) -> Self {
        Self { depth, email }
    }

    #[must_use]
    pub fn depth(&self) -> usize {
        self.depth
    }

    #[must_use]
    pub fn email(&self) -> &Email {
        &self.email
    }

    #[must_use]
    pub fn into_email(self) -> Email {
        self.email
    }
}

impl RenderTable for Vec<ThreadedEmail> {
    type Item = ThreadedEmail;

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
            .map(|threaded| {
                let email = &threaded.email;
                RowBuilder::new()
                    .with_cell(email.id.clone())
                    .with_cell_styled(
                        format!("{}{}", "  ".repeat(threaded.depth), email.subject),
                        subject_style(email),
                    )
                    .with_cell(email.from.address.clone())
                    .with_cell(email.to.address.clone())
                    .with_cell(email.date.to_rfc3339())
                    .with_cell_styled(
                        if email.has_attachment {
                            "Yes".to_string()
                        } else {
                            "No".to_string()
                        },
                        attachment_style(email),
                    )
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
        let emails: Vec<Email> = Vec::<Email>::from_headers_and_rows(headers, rows);

        emails
            .into_iter()
            .map(|email| {
                let depth = email.subject().chars().take_while(|c| *c == ' ').count() / 2;
                let subject = email.subject().trim_start().to_string();
                let email = Email::new(
                    email.id().to_string(),
                    email.flags().clone(),
                    subject,
                    email.from().clone(),
                    email.to().clone(),
                    *email.date(),
                    email.has_attachment(),
                );
                ThreadedEmail::new(depth, email)
            })
            .collect()
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
