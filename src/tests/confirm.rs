//! Integration tests for the confirmation popups ("user handholding"), run
//! inside a real (headless) Neovim via `#[nvim_oxi::test]`.

use std::sync::atomic::{AtomicBool, Ordering};

use nvim_oxi::Object;
use nvim_oxi::api::opts::OptionOpts;
use nvim_oxi::api::{self};

use crate::utils::confirm;

/// The confirmation popup buffers currently open.
fn confirm_popups() -> Vec<nvim_oxi::api::Buffer> {
    api::list_bufs()
        .filter(|buffer| {
            let opts = OptionOpts::builder().buf(buffer.clone()).build();
            api::get_option_value::<String>("filetype", &opts)
                .is_ok_and(|filetype| filetype == "mail-confirm")
        })
        .collect()
}

#[nvim_oxi::test]
fn confirm_yes_runs_the_pending_action_and_closes_the_popup() {
    static RAN: AtomicBool = AtomicBool::new(false);

    confirm::confirm(
        "Delete emails",
        vec!["Delete 1 email(s)?".to_string()],
        Box::new(|| {
            RAN.store(true, Ordering::SeqCst);
        }),
    );

    assert_eq!(confirm_popups().len(), 1, "expected a confirmation popup");
    assert!(
        !RAN.load(Ordering::SeqCst),
        "action must wait for confirmation"
    );

    // The popup's `y` binding calls `require('mail_nvim').confirm_yes()`.
    confirm::confirm_yes(Object::nil());

    assert!(RAN.load(Ordering::SeqCst), "expected the action to run");
    assert!(
        confirm_popups().is_empty(),
        "expected the popup to close after confirming"
    );
}

#[nvim_oxi::test]
fn confirm_no_discards_the_pending_action_and_closes_the_popup() {
    static RAN: AtomicBool = AtomicBool::new(false);

    confirm::confirm(
        "Delete emails",
        vec!["Delete 1 email(s)?".to_string()],
        Box::new(|| {
            RAN.store(true, Ordering::SeqCst);
        }),
    );

    assert_eq!(confirm_popups().len(), 1);

    confirm::confirm_no(Object::nil());

    assert!(!RAN.load(Ordering::SeqCst), "declined action must not run");
    assert!(
        confirm_popups().is_empty(),
        "expected the popup to close after declining"
    );
}

#[nvim_oxi::test]
fn a_new_confirmation_replaces_the_pending_one() {
    static FIRST: AtomicBool = AtomicBool::new(false);
    static SECOND: AtomicBool = AtomicBool::new(false);

    confirm::confirm(
        "Delete emails",
        vec!["Delete 1 email(s)?".to_string()],
        Box::new(|| {
            FIRST.store(true, Ordering::SeqCst);
        }),
    );
    confirm::confirm(
        "Delete emails",
        vec!["Delete 2 email(s)?".to_string()],
        Box::new(|| {
            SECOND.store(true, Ordering::SeqCst);
        }),
    );

    confirm::confirm_yes(Object::nil());

    assert!(
        !FIRST.load(Ordering::SeqCst),
        "replaced action must not run"
    );
    assert!(SECOND.load(Ordering::SeqCst), "latest action should run");
    assert!(confirm_popups().is_empty());
}
