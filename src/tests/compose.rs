//! Integration tests for composing and sending, run inside a real (headless)
//! Neovim via `#[nvim_oxi::test]`.
//!
//! `:MailEmailReply` (driven through `compose_with_config`) resolves the
//! email under the cursor, fetches it through the fake provider's async
//! pipeline and opens an editable compose buffer; `:MailEmailSend` (driven
//! through `send_with_config`) parses that buffer, builds a raw message and
//! sends it — closing the compose buffer on success.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use nvim_oxi::api::opts::{OptionOpts, OptionScope};
use nvim_oxi::api::{self, Buffer};

use crate::api::config::ui::view::{
    UiViewComponent, UiViewComponentContext, UiViewComponentContextContext, UiViewComponentType,
};
use crate::api::config::{Config, MailProvider, MailProviderType};
use crate::commands::email::compose::{
    ComposeKind, compose_create, compose_with_config, save_draft_with_config, send_with_config,
};
use crate::utils::buffer::metadata::BufferMetadata;
use crate::utils::buffer::render::FromBuffer;
use crate::utils::render::{create_base_buffer, load_into};

/// A config that uses the fake provider, like the other integration tests.
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

fn email_list_component() -> UiViewComponent {
    UiViewComponent {
        id: "compose-emails".into(),
        name: "Emails".into(),
        component_type: UiViewComponentType::List,
        context: UiViewComponentContext {
            command_group: "Email".into(),
            command_type: "List".into(),
            arguments: HashMap::new(),
            context: vec![
                UiViewComponentContextContext::AccountId("nic@example.com".into()),
                UiViewComponentContextContext::FolderId("INBOX".into()),
            ],
        },
        layout: None,
        on_enter: None,
        link: None,
    }
}

/// Loads the fake inbox through the async pipeline, parks the cursor on the
/// first email and returns the buffer.
fn load_inbox() -> Buffer {
    let opts = OptionOpts::builder().scope(OptionScope::Local).build();
    let buffer = create_base_buffer(&opts).expect("buffer should be created");
    load_into(email_list_component(), fake_config(), buffer.clone(), None);

    api::command("call wait(5000, {-> join(getline(1, '$'), \"\\n\") =~ 'sender1'})")
        .expect("wait should run");

    let metadata = BufferMetadata::from_buffer(&buffer, None).expect("metadata should parse");
    let first_data_row = metadata.line_count + 3; // metadata + header + separator
    let _ = api::get_current_win().set_cursor(first_data_row, 0);

    buffer
}

fn joined(buffer: &Buffer) -> String {
    buffer
        .get_lines(.., true)
        .expect("lines should be readable")
        .map(|line| line.to_string())
        .collect::<Vec<String>>()
        .join("\n")
}

#[nvim_oxi::test]
fn create_opens_an_empty_compose_buffer_for_the_current_account() {
    let config = fake_config();
    let list = load_inbox();

    compose_create(&config);

    let buffer = api::get_current_buf();
    assert_ne!(
        buffer.handle(),
        list.handle(),
        "the compose buffer must be current"
    );

    let content = joined(&buffer);
    assert!(content.contains("To: "), "expected an empty To line, got: {content}");
    assert!(
        content.contains("Subject: "),
        "expected an empty Subject line, got: {content}"
    );
    // No original message to quote.
    assert!(!content.contains("> "), "expected no quoted body, got: {content}");
    // The sending account comes from the buffer the command ran in.
    assert!(
        content.contains("nic@example.com"),
        "expected the account context, got: {content}"
    );
}

#[nvim_oxi::test]
fn create_falls_back_to_the_default_account() {
    // The fresh test buffer has no mail metadata: the default account is
    // used instead.
    let config = fake_config();
    compose_create(&config);

    let content = joined(&api::get_current_buf());
    assert!(content.contains("To: "), "expected a compose buffer, got: {content}");
    assert!(
        content.contains("nic@example.com"),
        "expected the default account, got: {content}"
    );
}

#[nvim_oxi::test]
fn save_draft_keeps_an_incomplete_message_and_closes_the_compose_buffer() {
    let config = fake_config();
    let list = load_inbox();

    // A brand-new (empty) compose buffer: drafts may be saved before the
    // recipients or subject are filled in.
    compose_create(&config);
    let compose = api::get_current_buf();
    assert_ne!(compose.handle(), list.handle());
    let compose_handle = compose.handle();

    let start = Instant::now();
    save_draft_with_config(config);

    // The fake provider delays ~0.5s before answering; on success the
    // compose buffer is deleted.
    api::command(&format!("call wait(5000, {{-> !bufexists({compose_handle})}})"))
        .expect("wait should run");
    assert!(
        start.elapsed() >= Duration::from_millis(450),
        "expected the fake 0.5s delay, took {:?}",
        start.elapsed()
    );

    let still_exists: i64 = api::eval(&format!("bufexists({compose_handle})"))
        .expect("bufexists should evaluate");
    assert_eq!(
        still_exists, 0,
        "expected the compose buffer to be deleted after saving the draft"
    );
}

#[nvim_oxi::test]
fn reply_opens_a_prefilled_compose_buffer() {
    let config = fake_config();
    let list = load_inbox();

    let start = Instant::now();
    compose_with_config(ComposeKind::Reply, config.clone());

    // The fake provider delays ~0.5s before answering, then the compose
    // buffer is opened on the main thread.
    api::command(
        "call wait(5000, {-> join(getline(1, '$'), \"\\n\") =~ 'Subject: Re: Re: Mail UI design'})",
    )
    .expect("wait should run");
    assert!(
        start.elapsed() >= Duration::from_millis(450),
        "expected the fake 0.5s delay, took {:?}",
        start.elapsed()
    );

    let buffer = api::get_current_buf();
    assert_ne!(
        buffer.handle(),
        list.handle(),
        "the compose buffer must be current"
    );

    let content = joined(&buffer);
    assert!(content.contains("To: Sender 1 <sender1@example.com>"), "{content}");
    // A subject already prefixed with `Re:` is not doubled.
    assert!(content.contains("Subject: Re: Re: Mail UI design (1)"), "{content}");
    // The original message is quoted.
    assert!(content.contains("> This is the fake body of email 1."), "{content}");
    assert!(content.contains("On "), "expected an attribution line, got: {content}");
}

#[nvim_oxi::test]
fn forward_opens_an_empty_recipient_compose_buffer() {
    let config = fake_config();
    let _list = load_inbox();

    compose_with_config(ComposeKind::Forward, config);

    api::command("call wait(5000, {-> join(getline(1, '$'), \"\\n\") =~ 'Fwd:'})")
        .expect("wait should run");

    let content = joined(&api::get_current_buf());
    assert!(
        content.contains("Subject: Fwd: Re: Re: Mail UI design (1)"),
        "{content}"
    );
    assert!(
        content.contains("---------- Forwarded message ----------"),
        "{content}"
    );
}

#[nvim_oxi::test]
fn send_delivers_the_message_and_closes_the_compose_buffer() {
    let config = fake_config();
    let list = load_inbox();

    compose_with_config(ComposeKind::Reply, config.clone());
    api::command(
        "call wait(5000, {-> join(getline(1, '$'), \"\\n\") =~ 'Subject: Re: Re: Mail UI design'})",
    )
    .expect("wait should run");

    let compose = api::get_current_buf();
    assert_ne!(compose.handle(), list.handle());
    let compose_handle = compose.handle();

    let start = Instant::now();
    send_with_config(config);

    // The fake provider delays ~0.5s before answering; on success the
    // compose buffer is deleted, so the current buffer switches back to the
    // email list.
    api::command(&format!("call wait(5000, {{-> !bufexists({compose_handle})}})"))
        .expect("wait should run");
    let still_exists: i64 = api::eval(&format!("bufexists({compose_handle})"))
        .expect("bufexists should evaluate");
    assert_eq!(
        still_exists, 0,
        "expected the compose buffer to be deleted after sending"
    );
    assert!(
        start.elapsed() >= Duration::from_millis(450),
        "expected the fake 0.5s delay, took {:?}",
        start.elapsed()
    );

    assert!(
        !compose.is_valid(),
        "expected the compose buffer to be closed after sending"
    );
}
