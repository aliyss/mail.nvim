//! Integration tests for the fake provider, run inside a real (headless)
//! Neovim via `#[nvim_oxi::test]`.
//!
//! These tests drive the *real* async pipeline — `load_into` spawns the fetch
//! on the tokio runtime, the provider answers after the fake network delay,
//! and the result is scheduled back to the main thread and rendered — so the
//! exact same route a real Himalaya account takes is exercised, without any
//! network. If the pipeline crashes (or deadlocks, or never renders), these
//! tests reproduce it.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use nvim_oxi::api::opts::{OptionOpts, OptionScope};
use nvim_oxi::api::{self, Buffer};

use crate::api::account::Account;
use crate::api::config::{MailProvider, MailProviderType};
use crate::api::config::ui::view::{
    UiViewComponent, UiViewComponentContext, UiViewComponentContextContext, UiViewComponentType,
};
use crate::api::config::Config;
use crate::commands::ui::drawer::{open_mail_list, render_tree, test_set_accounts, toggle_account};
use crate::commands::ui::setup_drawer_buffer;
use crate::utils::buffer::metadata::BufferMetadata;
use crate::utils::buffer::render::FromBuffer;
use crate::utils::loading::{self, Anchor, FRAMES};
use crate::utils::render::table::render::Table;
use crate::utils::render::{
    ASYNC_RUNTIME, ComponentData, create_base_buffer, get_data, load_into, render_into_buffer,
};

/// A config that uses the fake provider, exactly like a real one but backed
/// by in-memory data.
fn fake_config() -> Config {
    Config::builder()
        .mail_provider(
            MailProvider::builder()
                .provider_type(MailProviderType::Fake)
                .build()
                .expect("fake provider config"),
        )
        .build()
        .expect("fake config")
}

fn component(
    command_group: &str,
    command_type: &str,
    context: Vec<UiViewComponentContextContext>,
) -> UiViewComponent {
    UiViewComponent {
        id: format!("fake-{command_group}-{command_type}"),
        name: format!("{command_group}{command_type}"),
        // Threads render as tables (like the real `MailEmailThread`), messages
        // as file views.
        component_type: match command_type {
            "List" | "Thread" => UiViewComponentType::Table,
            _ => UiViewComponentType::File,
        },
        context: UiViewComponentContext {
            command_group: command_group.into(),
            command_type: command_type.into(),
            arguments: HashMap::new(),
            context,
        },
        layout: None,
        on_enter: None,
        link: None,
    }
}

fn account_list_component() -> UiViewComponent {
    component("Account", "List", Vec::new())
}

fn folder_list_component(account: &str) -> UiViewComponent {
    component(
        "Folder",
        "List",
        vec![UiViewComponentContextContext::AccountId(account.into())],
    )
}

fn email_list_component(account: &str, folder: &str) -> UiViewComponent {
    component(
        "Email",
        "List",
        vec![
            UiViewComponentContextContext::AccountId(account.into()),
            UiViewComponentContextContext::FolderId(folder.into()),
        ],
    )
}

fn email_get_component(account: &str, folder: &str, email: &str) -> UiViewComponent {
    component(
        "Email",
        "Get",
        vec![
            UiViewComponentContextContext::AccountId(account.into()),
            UiViewComponentContextContext::FolderId(folder.into()),
            UiViewComponentContextContext::EmailId(email.into()),
        ],
    )
}

fn thread_component(account: &str, folder: &str, email: &str) -> UiViewComponent {
    component(
        "Email",
        "Thread",
        vec![
            UiViewComponentContextContext::AccountId(account.into()),
            UiViewComponentContextContext::FolderId(folder.into()),
            UiViewComponentContextContext::EmailId(email.into()),
        ],
    )
}

/// Pumps Neovim's event loop until `condition` (a vimscript expression over
/// the current buffer) becomes true, or `timeout_ms` elapse.
fn wait_for_condition(condition: &str, timeout_ms: i64) {
    let script = format!("call wait({timeout_ms}, {{-> {condition}}})");
    api::command(&script).expect("wait should run");
}

/// The current contents of `buffer`, line by line.
fn buffer_lines(buffer: &Buffer) -> Vec<String> {
    buffer
        .get_lines(.., true)
        .expect("lines should be readable")
        .map(|line| line.to_string())
        .collect()
}

fn joined(buffer: &Buffer) -> String {
    buffer_lines(buffer).join("\n")
}

fn new_buffer() -> Buffer {
    let opts = OptionOpts::builder().scope(OptionScope::Local).build();
    create_base_buffer(&opts).expect("buffer should be created")
}

/// The 1-indexed buffer line of the first data row of a rendered list table.
fn first_data_row(buffer: &Buffer) -> usize {
    let metadata = BufferMetadata::from_buffer(buffer, None).expect("metadata should parse");
    let table =
        Table::<Vec<crate::api::email::Email>>::from_buffer(buffer, Some(metadata.line_count))
            .expect("table parses");
    table.offset + 1
}

fn has_spinner(line: &str) -> bool {
    FRAMES.iter().any(|frame| line.contains(*frame))
}

#[nvim_oxi::test]
fn account_list_comes_from_the_fake_provider() {
    let config = fake_config();
    let component = account_list_component();

    let data = ASYNC_RUNTIME
        .block_on(get_data(&component, &config))
        .expect("get_data should succeed");

    let ComponentData::Accounts(accounts) = data else {
        panic!("expected accounts, got something else");
    };
    assert!(
        accounts.iter().any(|account| account.name() == "nic@example.com"),
        "expected the fake accounts"
    );
}

#[nvim_oxi::test]
fn folder_list_renders_through_the_async_pipeline() {
    let config = fake_config();
    let component = folder_list_component("nic@example.com");
    let buffer = new_buffer();

    let start = Instant::now();
    load_into(component, config, buffer.clone(), None);

    // The fake provider sleeps ~0.5s before answering, then the result is
    // scheduled back to the main thread and rendered — the real route.
    wait_for_condition("join(getline(1, '$'), \"\\n\") =~ 'INBOX'", 5000);
    let elapsed = start.elapsed();

    assert!(
        elapsed >= Duration::from_millis(450),
        "expected the fake 0.5s delay, took {elapsed:?}"
    );
    let joined = joined(&buffer);
    assert!(joined.contains("INBOX"), "expected the fake folders");
    assert!(joined.contains("nic@example.com"), "expected the account context");
}

#[nvim_oxi::test]
fn email_list_renders_through_the_async_pipeline() {
    let config = fake_config();
    let component = email_list_component("nic@example.com", "INBOX");
    let buffer = new_buffer();

    // A wide screen so the table columns fit without truncating the sender.
    api::command("set columns=240").expect("wide screen");

    let start = Instant::now();
    load_into(component, config, buffer.clone(), None);

    wait_for_condition("join(getline(1, '$'), \"\\n\") =~ 'Invoice'", 5000);
    let elapsed = start.elapsed();

    assert!(
        elapsed >= Duration::from_millis(450),
        "expected the fake 0.5s delay, took {elapsed:?}"
    );
    let joined = joined(&buffer);
    assert!(joined.contains("Invoice"), "expected the fake emails, got:\n{joined}");
    // The `From` column can be truncated by the narrow pane, but the sender
    // prefix survives.
    assert!(joined.contains("sender1"), "expected the fake senders, got:\n{joined}");
}

#[nvim_oxi::test]
fn email_message_renders_through_the_async_pipeline() {
    let config = fake_config();
    let component = email_get_component("nic@example.com", "INBOX", "1");
    let buffer = new_buffer();

    load_into(component, config, buffer.clone(), None);

    wait_for_condition("join(getline(1, '$'), \"\\n\") =~ 'fake body'", 5000);

    let joined = joined(&buffer);
    assert!(joined.contains("fake body of email 1"), "expected the fake message");
}

#[nvim_oxi::test]
fn thread_renders_through_the_async_pipeline() {
    let config = fake_config();
    let component = thread_component("nic@example.com", "INBOX", "3");
    let buffer = new_buffer();

    load_into(component, config, buffer.clone(), None);

    // The table truncates long subjects with an ellipsis, so match the
    // subject prefix without the trailing space.
    wait_for_condition("join(getline(1, '$'), \"\\n\") =~ 'Re:'", 5000);

    let joined = joined(&buffer);
    assert!(joined.contains("3.1"), "expected the threaded reply, got:\n{joined}");
    assert!(joined.contains("Re:"), "expected the threaded subjects, got:\n{joined}");
}

#[nvim_oxi::test]
fn spinner_shows_while_the_fake_provider_loads_and_clears_after() {
    let config = fake_config();
    let component = email_list_component("nic@example.com", "INBOX");

    // Render stale rows first so the loading row has a home in the table.
    let mut buffer = new_buffer();
    render_into_buffer(
        &mut buffer,
        &component,
        ComponentData::Emails(vec![]),
    )
    .expect("stale render should succeed");

    // A stale email list with no rows gives the spinner nothing to anchor to;
    // render one fake row directly so the Anchor::Row(0) resolves.
    render_into_buffer(
        &mut buffer,
        &component,
        ComponentData::Emails(vec![crate::api::email::Email::new(
            "stale".into(),
            std::collections::HashSet::new(),
            "stale subject".into(),
            crate::api::email::Mailbox {
                name: None,
                address: "stale@example.com".into(),
            },
            crate::api::email::Mailbox {
                name: None,
                address: "stale@example.com".into(),
            },
            chrono::Utc::now(),
            false,
        )]),
    )
    .expect("stale render should succeed");

    let row = first_data_row(&buffer);

    // Trigger a real load with a spinner guard on the first row.
    let guard = loading::Guard::new(buffer.clone(), Anchor::Row(0));
    let mut buffer_for_render = buffer.clone();
    render_into_buffer(
        &mut buffer_for_render,
        &component,
        ComponentData::Emails(vec![crate::api::email::Email::new(
            "stale".into(),
            std::collections::HashSet::new(),
            "stale subject".into(),
            crate::api::email::Mailbox {
                name: None,
                address: "stale@example.com".into(),
            },
            crate::api::email::Mailbox {
                name: None,
                address: "stale@example.com".into(),
            },
            chrono::Utc::now(),
            false,
        )]),
    )
    .expect("render with spinner should succeed");

    // The spinner is drawn while the fetch is in flight.
    let line = buffer_lines(&buffer)[row - 1].clone();
    assert!(has_spinner(&line), "expected a spinner during the load, got: {line}");

    load_into(component, config, buffer.clone(), Some(guard));

    // The real data arrives after the fake delay; the spinner is cleared.
    wait_for_condition("join(getline(1, '$'), \"\\n\") =~ 'Invoice'", 5000);

    let joined = joined(&buffer);
    assert!(joined.contains("Invoice"), "expected the fake emails to render");
    assert!(
        !has_spinner(&joined),
        "expected the spinner to be cleared after the load"
    );
}

/// Sets up a drawer buffer with one fake account rendered into it.
fn setup_drawer() -> (Buffer, usize) {
    let mut buffer = new_buffer();
    setup_drawer_buffer(&mut buffer).expect("drawer buffer should be set up");

    test_set_accounts(vec![Account::new(
        "nic@example.com".into(),
        Some("imap".into()),
        true,
    )]);
    render_tree(&mut buffer).expect("drawer should render");

    // The first line after the metadata block is the account node.
    let metadata = BufferMetadata::from_buffer(&buffer, None).expect("metadata should parse");
    (buffer, metadata.line_count)
}

#[nvim_oxi::test]
fn drawer_expands_an_account_with_the_fake_provider() {
    let config = fake_config();
    let (buffer, account_index) = setup_drawer();

    // No spinner before the expansion is triggered.
    let line = buffer_lines(&buffer)[account_index].clone();
    assert!(!has_spinner(&line), "expected no spinner yet, got: {line}");

    // Expanding fetches the folders with the fake delay; the account node
    // shows a spinner while the fetch is in flight.
    toggle_account(&buffer, "nic@example.com", config);

    let line = buffer_lines(&buffer)[account_index].clone();
    assert!(
        has_spinner(&line),
        "expected a spinner on the expanding account, got: {line}"
    );

    // The folders arrive after ~0.5s and the tree expands in place.
    wait_for_condition("join(getline(1, '$'), \"\\n\") =~ 'INBOX'", 5000);

    let joined = joined(&buffer);
    assert!(joined.contains("▾ nic@example.com"), "expected the expanded account");
    assert!(joined.contains("INBOX"), "expected the fake folders, got:\n{joined}");
    assert!(
        !has_spinner(&joined),
        "expected the spinner to be cleared after the folders arrived"
    );

    test_set_accounts(Vec::new());
}

#[nvim_oxi::test]
fn drawer_action_opens_the_mail_list_with_the_fake_provider() {
    let config = fake_config();
    let (_buffer, _) = setup_drawer();

    // A wide screen so the pane that opens to the right can show the full
    // subjects.
    api::command("set columns=240").expect("columns should resize");

    // Trigger the "List Mail" action: a pane opens to the right and loads
    // the fake emails of the folder. The pane height caps the page at one
    // row, so the first fake email (subject "Re: Mail UI design", sender
    // sender1) is what shows up.
    open_mail_list("nic@example.com", "INBOX", config);

    wait_for_condition("join(getline(1, '$'), \"\\n\") =~ 'sender1'", 5000);

    let joined = joined(&api::get_current_buf());
    assert!(
        joined.contains("sender1"),
        "expected the mail list to load in the new pane, got:\n{joined}"
    );

    test_set_accounts(Vec::new());
}

/// The `ui_enter` spinner row for a cursor on the `n`-th data row, computed
/// exactly like `ui_enter` does (data row `i` sits at 1-indexed buffer line
/// `metadata_line_count + 3 + i`).
fn enter_spinner_row(cursor_row: usize, metadata_line_count: usize) -> usize {
    cursor_row.saturating_sub(metadata_line_count + 3)
}

/// Replicates what `:MailAccountList` + `<CR>` does: drills from the account
/// list into the folder list, then into the email list, then opens an email,
/// every step through `load_into` with a loading-spinner guard on the source
/// row (the exact `ui_enter` → `replace_current` path).
#[nvim_oxi::test]
fn drill_down_account_folder_email_through_the_async_pipeline() {
    let config = fake_config();
    api::command("set columns=240").expect("wide screen");

    // 1. The account list loads.
    let mut buffer = new_buffer();
    let account_component = account_list_component();
    load_into(account_component.clone(), config.clone(), buffer.clone(), None);
    wait_for_condition("join(getline(1, '$'), \"\\n\") =~ 'nic@example.com'", 5000);

    // 2. <CR> on the first account: spinner guard on its row, then the pane
    // is replaced by the folder list.
    let metadata = BufferMetadata::from_buffer(&buffer, None).expect("metadata should parse");
    let first_data_row = metadata.line_count + 3; // 1-indexed
    let _ = api::get_current_win().set_cursor(first_data_row, 0);

    let row = enter_spinner_row(first_data_row, metadata.line_count);
    assert_eq!(row, 0, "the spinner anchor must be the cursor's data row");
    let guard = loading::Guard::new(buffer.clone(), Anchor::Row(row));
    let data = crate::utils::render::cached_pane_data(&buffer).expect("cached accounts");
    render_into_buffer(&mut buffer, &account_component, data).expect("re-render with spinner");

    // The spinner is drawn on the row under the cursor (the first data row), not below it.
    let spinner_line = buffer_lines(&buffer)[metadata.line_count + 2].clone();
    assert!(has_spinner(&spinner_line), "expected the spinner on the cursor's row");

    let folders_component = folder_list_component("nic@example.com");
    load_into(folders_component.clone(), config.clone(), buffer.clone(), Some(guard));
    wait_for_condition("join(getline(1, '$'), \"\\n\") =~ 'INBOX'", 5000);

    // 3. <CR> on the first folder: spinner guard, then the pane is replaced
    // by the email list.
    let metadata = BufferMetadata::from_buffer(&buffer, None).expect("metadata should parse");
    let first_data_row = metadata.line_count + 3;
    let _ = api::get_current_win().set_cursor(first_data_row, 0);

    let row = enter_spinner_row(first_data_row, metadata.line_count);
    let guard = loading::Guard::new(buffer.clone(), Anchor::Row(row));
    let data = crate::utils::render::cached_pane_data(&buffer).expect("cached folders");
    render_into_buffer(&mut buffer, &folders_component, data).expect("re-render with spinner");

    let emails_component = email_list_component("nic@example.com", "INBOX");
    load_into(emails_component.clone(), config.clone(), buffer.clone(), Some(guard));
    wait_for_condition("join(getline(1, '$'), \"\\n\") =~ 'sender1'", 5000);

    // 4. <CR> on the first email: spinner guard, then the pane is replaced
    // by the message view.
    let metadata = BufferMetadata::from_buffer(&buffer, None).expect("metadata should parse");
    let first_data_row = metadata.line_count + 3;
    let _ = api::get_current_win().set_cursor(first_data_row, 0);

    let row = enter_spinner_row(first_data_row, metadata.line_count);
    let guard = loading::Guard::new(buffer.clone(), Anchor::Row(row));
    let data = crate::utils::render::cached_pane_data(&buffer).expect("cached emails");
    render_into_buffer(&mut buffer, &emails_component, data).expect("re-render with spinner");

    let message_component = email_get_component("nic@example.com", "INBOX", "1");
    load_into(message_component, config, buffer.clone(), Some(guard));
    wait_for_condition("join(getline(1, '$'), \"\\n\") =~ 'fake body'", 5000);

    let joined = joined(&buffer);
    assert!(
        joined.contains("fake body of email 1"),
        "expected the message to load, got:\n{joined}"
    );
}
