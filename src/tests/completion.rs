//! Integration tests for live command-line completion, run inside a real
//! (headless) Neovim via `#[nvim_oxi::test]`.
//!
//! The commands registered by the plugin are not available in the test
//! harness, so each test registers the command it exercises before asking
//! Neovim for the completion candidates.

use nvim_oxi::api::{self};

use crate::api::folder::Folder;
use crate::commands::UserCommand;
use crate::commands::config::settings::user_handholding::UserHandHoldingSwitchOn;
use crate::commands::email::manage::EmailFlagAdd;
use crate::commands::email::manage::EmailMove;
use crate::utils::completion;

/// The completion candidates Neovim offers for `cmdline`.
fn completes(cmdline: &str) -> Vec<String> {
    let mut results: Vec<String> =
        api::call_function("getcompletion", (cmdline, "cmdline")).unwrap_or_default();
    results.sort();
    results.dedup();
    results
}

#[nvim_oxi::test]
fn flag_arguments_complete_dynamically() {
    EmailFlagAdd::register().expect("failed to register MailEmailFlagAdd");

    // Typing `fl` completes to `flagged`, nothing else.
    assert_eq!(
        completes("MailEmailFlagAdd fl"),
        vec!["flagged".to_string()]
    );

    // An empty lead offers every known flag.
    assert_eq!(
        completes("MailEmailFlagAdd "),
        vec![
            "answered".to_string(),
            "deleted".to_string(),
            "draft".to_string(),
            "flagged".to_string(),
            "seen".to_string(),
        ]
    );
}

#[nvim_oxi::test]
fn boolean_arguments_complete() {
    UserHandHoldingSwitchOn::register()
        .expect("failed to register MailConfigUserHandHoldingSwitchOn");

    assert!(completes("MailConfigUserHandHoldingSwitchOn t").contains(&"true".to_string()));
    assert!(completes("MailConfigUserHandHoldingSwitchOn ").contains(&"false".to_string()));
}

#[nvim_oxi::test]
fn folder_arguments_complete_from_the_cache() {
    // Populate the folder cache the way a folder listing would.
    completion::cache_folders(
        "acc",
        vec![
            Folder::new("INBOX".into(), None, None, None, true),
            Folder::new("Trash".into(), None, None, None, false),
        ],
    );

    EmailMove::register().expect("failed to register MailEmailMove");

    assert_eq!(completes("MailEmailMove Tr"), vec!["Trash".to_string()]);
    assert_eq!(
        completes("MailEmailMove "),
        vec!["INBOX".to_string(), "Trash".to_string()]
    );
}
