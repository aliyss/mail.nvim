//! A provider that serves deterministic fake data through the same async
//! pipeline as [`super::himalaya::HimalayaProvider`].
//!
//! It exists so the Mail UI can be exercised (and tested) without a real mail
//! account: every asynchronous call takes the exact same route as a real
//! backend (spawned on the async runtime, awaited, rendered back on the main
//! thread) but answers from in-memory data after a short fake network delay.
//! Configure it with `MailProviderType::Fake`.

use std::collections::HashSet;
use std::time::Duration;

use chrono::{DateTime, Days, Utc};

use crate::api::account::Account;
use crate::api::contact::{Address, Contact};
use crate::api::email::arguments::EmailListArguments;
use crate::api::email::{Email, EmailFlag, EmailMessage, Mailbox, ThreadedEmail};
use crate::api::folder::{Folder, FolderFlag};

mod account;
mod email;
mod folder;

/// How long each fake provider call "waits", mimicking the round trip to a
/// real mail server.
pub const FAKE_DELAY: Duration = Duration::from_millis(500);

/// The provider behind `MailProviderType::Fake`.
#[derive(Debug, Clone, Default)]
pub struct FakeProvider;

/// The fake network round trip every asynchronous provider method waits
/// through before answering.
pub(super) async fn fake_delay() {
    tokio::time::sleep(FAKE_DELAY).await;
}

/// The accounts the fake provider serves, in display order.
pub(super) fn accounts() -> Vec<Account> {
    vec![
        Account::new("nic@example.com".into(), Some("imap".into()), true),
        Account::new("bob@example.com".into(), Some("imap".into()), false),
    ]
}

/// The folders of `account_id`. Every account gets the same tree.
pub(super) fn folders(account_id: &str) -> Vec<Folder> {
    let _ = account_id;

    vec![
        Folder::new(
            "INBOX".into(),
            None,
            Some("\\HasChildren".into()),
            Some(vec![FolderFlag::from("\\HasChildren".to_string())]),
            true,
        ),
        Folder::new("INBOX.Sent".into(), None, Some("\\HasNoChildren".into()), None, false),
        Folder::new("INBOX.Drafts".into(), None, Some("\\Drafts".into()), None, false),
        Folder::new("Archive".into(), None, Some("\\Archive".into()), None, false),
        Folder::new("Trash".into(), None, Some("\\Trash".into()), None, false),
        Folder::new("Snoozed".into(), None, Some("\\Snoozed".into()), None, false),
    ]
}

/// How many emails the fake provider can list per folder.
const FAKE_EMAIL_COUNT: usize = 12;

/// Deterministic subjects, cycled through the fake emails.
const SUBJECTS: [&str; 6] = [
    "Re: Mail UI design",
    "Invoice #1234",
    "Weekly standup notes",
    "Himalaya release announcement",
    "Lunch on Friday?",
    "Meeting moved to 3pm",
];

/// A mailbox for a fake sender with the given index.
fn sender(index: usize) -> Mailbox {
    Mailbox {
        name: Some(format!("Sender {index}")),
        address: format!("sender{index}@example.com"),
    }
}

/// A mailbox for the recipient of a fake email.
fn recipient() -> Mailbox {
    Mailbox {
        name: None,
        address: "recipient@example.com".into(),
    }
}

/// The flags of the fake email `index`: odd ones are seen, every third one
/// flagged, the fifth one answered.
fn flags(index: usize) -> HashSet<EmailFlag> {
    let mut flags = HashSet::new();
    if index % 2 == 1 {
        flags.insert(EmailFlag::Seen);
    }
    if index.is_multiple_of(3) {
        flags.insert(EmailFlag::Flagged);
    }
    if index == 5 {
        flags.insert(EmailFlag::Answered);
    }
    flags
}

/// A fixed point in time every fake email is dated relative to.
fn base_date() -> DateTime<Utc> {
    // An arbitrary, stable instant so the fake data renders the same in every
    // test run.
    DateTime::parse_from_rfc3339("2026-01-15T09:30:00Z")
        .expect("static date should parse")
        .with_timezone(&Utc)
}

/// Builds the fake email with the given (1-based) index.
fn email(index: usize) -> Email {
    let id = index.to_string();
    let date = base_date()
        .checked_sub_days(Days::new(u64::try_from(index).unwrap_or(u64::MAX)))
        .unwrap_or_else(base_date);

    Email::new(
        id,
        flags(index),
        SUBJECTS[(index - 1) % SUBJECTS.len()].to_string(),
        sender(index),
        recipient(),
        date,
        index.is_multiple_of(4),
    )
}

/// Lists the fake emails of a folder, honoring the pagination options like a
/// real backend would.
pub(super) fn emails(
    _account_id: &str,
    _folder_id: &str,
    options: Option<EmailListArguments>,
) -> Vec<Email> {
    let options = options.unwrap_or_default();
    let page = options.page_or_default();
    let per_page = options.per_page_or_default(None::<fn() -> usize>);

    (1..=FAKE_EMAIL_COUNT)
        .map(email)
        .skip(page.saturating_mul(per_page))
        .take(per_page)
        .collect()
}

/// A fake sender address with the given index.
fn address(index: usize) -> Address {
    Address::Individual(Contact {
        name: Some(format!("Sender {index}")),
        email: format!("sender{index}@example.com"),
    })
}

/// The full message of a fake email id. Every id is served, so thread
/// navigation can open the replies too.
pub(super) fn message(id: &str) -> EmailMessage {
    let index = id
        .split('.')
        .next()
        .and_then(|head| head.parse::<usize>().ok())
        .unwrap_or(1);

    EmailMessage {
        id: id.to_string(),
        thread_id: Some(id.to_string()),
        subject: format!("Re: {} ({id})", SUBJECTS[(index - 1) % SUBJECTS.len()]),
        from: vec![address(index)],
        to: vec![Address::Individual(Contact {
            name: None,
            email: "recipient@example.com".into(),
        })],
        cc: Vec::new(),
        bcc: Vec::new(),
        date: Some(
            base_date()
                .checked_sub_days(Days::new(u64::try_from(index).unwrap_or(u64::MAX)))
                .unwrap_or_else(base_date),
        ),
        body_text: format!(
            "This is the fake body of email {id}.\n\n\
             It is served by the fake provider, so the full async pipeline \
             can be exercised without a real mail account.\n\n\
             Regards,\nSender {index}"
        ),
        body_html: None,
        attachment_ids: if index.is_multiple_of(4) { vec!["1".into()] } else { Vec::new() },
    }
}

/// The thread of `email_id`: the email itself plus two replies, indented one
/// level under it.
pub(super) fn thread(
    account_id: &str,
    folder_id: &str,
    email_id: &str,
    options: Option<EmailListArguments>,
) -> Vec<ThreadedEmail> {
    let mut root: Vec<Email> = emails(account_id, folder_id, options)
        .into_iter()
        .filter(|email| email.id() == email_id)
        .collect();

    // The email may not be on the current page; serve it from the full set.
    if root.is_empty() {
        root = (1..=FAKE_EMAIL_COUNT)
            .map(email)
            .filter(|email| email.id() == email_id)
            .collect();
    }

    let Some(root) = root.into_iter().next() else {
        return Vec::new();
    };

    let mut out = vec![ThreadedEmail::new(0, root.clone())];

    for (offset, reply_id) in [format!("{email_id}.1"), format!("{email_id}.2")].into_iter().enumerate() {
        let reply = Email::new(
            reply_id,
            flags(offset + 1),
            format!("Re: {}", root.subject()),
            sender(offset + 2),
            recipient(),
            *root.date(),
            false,
        );
        out.push(ThreadedEmail::new(1, reply));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accounts_are_deterministic() {
        let accounts = accounts();
        assert_eq!(accounts.len(), 2);
        assert!(accounts[0].is_default());
        assert_eq!(accounts[0].name(), "nic@example.com");
        assert!(!accounts[1].is_default());
    }

    #[test]
    fn folders_contain_the_inbox() {
        let folders = folders("nic@example.com");
        assert!(folders.iter().any(|folder| folder.id() == "INBOX"));
        assert!(folders[0].has_children());
    }

    #[test]
    fn emails_are_deterministic_and_paginated() {
        let all = emails(
            "nic@example.com",
            "INBOX",
            Some(EmailListArguments::new(Some(1), Some(100))),
        );
        assert_eq!(all.len(), FAKE_EMAIL_COUNT);

        let first_page = emails("nic@example.com", "INBOX", None);
        assert_eq!(first_page.len(), 10, "default page size is 10");
        assert_eq!(first_page[0].id(), "1");
        assert_eq!(first_page[9].id(), "10");

        let second_page = emails(
            "nic@example.com",
            "INBOX",
            Some(EmailListArguments::new(Some(2), Some(10))),
        );
        assert_eq!(second_page.len(), 2);
        assert_eq!(second_page[0].id(), "11");
    }

    #[test]
    fn message_of_any_id_is_served() {
        let message = message("42");
        assert_eq!(message.id, "42");
        assert!(message.body_text.contains("fake body"));
    }

    #[test]
    fn thread_contains_the_email_and_replies() {
        let thread = thread("nic@example.com", "INBOX", "3", None);
        assert_eq!(thread.len(), 3);
        assert_eq!(thread[0].depth(), 0);
        assert_eq!(thread[0].email().id(), "3");
        assert_eq!(thread[1].depth(), 1);
        assert_eq!(thread[2].depth(), 1);
    }
}
