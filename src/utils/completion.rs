//! Cache of recently fetched provider data, used for live command-line
//! completion.
//!
//! Completing values like folder or email ids would require a network
//! round-trip, which cannot happen synchronously while Neovim is completing
//! the command line. Instead, every time a folder or email list is fetched
//! (see [`crate::utils::render::get_data`]) its ids are cached here, so the
//! completion can offer them instantly — reflecting whatever the user has
//! actually loaded so far.

use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

use crate::api::email::Email;
use crate::api::folder::Folder;

/// Folder ids per account, in fetch order.
static FOLDERS: LazyLock<RwLock<HashMap<String, Vec<String>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Email ids per `(account, folder)` pair.
static EMAILS: LazyLock<RwLock<HashMap<(String, String), Vec<String>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Caches the folder ids of `account`.
pub(crate) fn cache_folders(account: &str, folders: Vec<Folder>) {
    let ids = folders
        .into_iter()
        .map(|folder| folder.id().to_string())
        .collect();
    FOLDERS
        .write()
        .expect("folder cache lock poisoned")
        .insert(account.to_string(), ids);
}

/// Caches the email ids of `account`/`folder`.
pub(crate) fn cache_emails(account: &str, folder: &str, emails: Vec<Email>) {
    let ids = emails
        .into_iter()
        .map(|email| email.id().to_string())
        .collect();
    EMAILS
        .write()
        .expect("email cache lock poisoned")
        .insert((account.to_string(), folder.to_string()), ids);
}

/// The cached folder ids of `account`, or of every account when `account` is
/// `None`.
#[must_use]
pub fn folder_names(account: Option<&str>) -> Vec<String> {
    let cache = FOLDERS.read().expect("folder cache lock poisoned");
    match account {
        Some(account) => cache.get(account).cloned().unwrap_or_default(),
        None => cache.values().flatten().cloned().collect(),
    }
}

/// The cached email ids of `account`, in `folder` when given or across every
/// cached folder of the account otherwise.
#[must_use]
pub fn email_ids(account: &str, folder: Option<&str>) -> Vec<String> {
    let cache = EMAILS.read().expect("email cache lock poisoned");
    match folder {
        Some(folder) => cache
            .get(&(account.to_string(), folder.to_string()))
            .cloned()
            .unwrap_or_default(),
        None => cache
            .iter()
            .filter(|((cached_account, _), _)| cached_account == account)
            .flat_map(|(_, ids)| ids.iter().cloned())
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_cache_is_per_account() {
        cache_folders(
            "acc-a",
            vec![Folder::new("INBOX".into(), None, None, None, true)],
        );
        cache_folders(
            "acc-b",
            vec![
                Folder::new("INBOX".into(), None, None, None, true),
                Folder::new("Trash".into(), None, None, None, false),
            ],
        );

        assert_eq!(folder_names(Some("acc-a")), vec!["INBOX"]);
        assert_eq!(folder_names(Some("acc-b")), vec!["INBOX", "Trash"]);

        let mut all = folder_names(None);
        all.sort();
        assert_eq!(all, vec!["INBOX", "INBOX", "Trash"]);
    }

    #[test]
    fn email_cache_is_per_account_and_folder() {
        cache_emails("acc", "INBOX", vec![email("1"), email("2")]);
        cache_emails("acc", "Trash", vec![email("3")]);
        cache_emails("other", "INBOX", vec![email("9")]);

        assert_eq!(email_ids("acc", Some("INBOX")), vec!["1", "2"]);
        assert_eq!(email_ids("acc", Some("Trash")), vec!["3"]);

        let mut all = email_ids("acc", None);
        all.sort();
        assert_eq!(all, vec!["1", "2", "3"]);
        assert_eq!(email_ids("missing", None), Vec::<String>::new());
    }

    fn email(id: &str) -> Email {
        use crate::api::email::Mailbox;
        use chrono::Utc;
        use std::collections::HashSet;

        Email::new(
            id.to_string(),
            HashSet::new(),
            "Subject".into(),
            Mailbox {
                name: None,
                address: "a@b.c".into(),
            },
            Mailbox {
                name: None,
                address: "d@e.f".into(),
            },
            Utc::now(),
            false,
        )
    }
}
