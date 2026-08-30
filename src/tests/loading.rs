//! Integration tests for the loading spinner, run inside a real (headless)
//! Neovim via `#[nvim_oxi::test]`.

use std::collections::HashMap;

use chrono::Utc;
use nvim_oxi::api::opts::{OptionOpts, OptionScope};
use nvim_oxi::api::{self, Buffer};

use crate::api::account::Account;
use crate::api::config::ui::view::{
    UiViewComponent, UiViewComponentContext, UiViewComponentContextContext, UiViewComponentType,
};
use crate::api::email::{Email, Mailbox};
use crate::commands::ui::drawer::{render_tree, test_set_accounts};
use crate::commands::ui::setup_drawer_buffer;
use crate::utils::buffer::metadata::BufferMetadata;
use crate::utils::buffer::render::FromBuffer;
use crate::utils::loading::{self, Anchor, FRAMES};
use crate::utils::render::table::render::Table;
use crate::utils::render::{ComponentData, create_base_buffer, render_into_buffer};

fn email(id: &str) -> Email {
    Email::new(
        id.to_string(),
        std::collections::HashSet::new(),
        format!("Subject {id}"),
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

fn component(command_group: &str, command_type: &str) -> UiViewComponent {
    UiViewComponent {
        id: "test-component".into(),
        name: "Test".into(),
        component_type: UiViewComponentType::List,
        context: UiViewComponentContext {
            command_group: command_group.into(),
            command_type: command_type.into(),
            arguments: HashMap::new(),
            context: vec![
                UiViewComponentContextContext::AccountId("acc".into()),
                UiViewComponentContextContext::FolderId("INBOX".into()),
            ],
        },
        layout: None,
        on_enter: None,
        link: None,
    }
}

fn render_component(buffer: &mut Buffer, component: &UiViewComponent, data: ComponentData) {
    render_into_buffer(buffer, component, data).expect("component should render");
}

/// Returns the 1-indexed `line` of `buffer`.
fn buffer_line(buffer: &Buffer, line: usize) -> String {
    buffer
        .get_lines(line - 1..line, true)
        .expect("line should be readable")
        .map(|s| s.to_string())
        .next()
        .unwrap_or_default()
}

/// The 1-indexed buffer line of the first data row of a rendered list table.
fn first_data_row(buffer: &Buffer) -> usize {
    let metadata = BufferMetadata::from_buffer(buffer, None).expect("metadata should parse");
    let table =
        Table::<Vec<Email>>::from_buffer(buffer, Some(metadata.line_count)).expect("table parses");
    table.offset + 1
}

fn has_spinner(line: &str) -> bool {
    FRAMES.iter().any(|frame| line.contains(*frame))
}

#[nvim_oxi::test]
fn email_list_row_shows_spinner_until_cleared() {
    let component = component("Email", "List");
    let opts = OptionOpts::builder().scope(OptionScope::Local).build();
    let mut buffer = create_base_buffer(&opts).expect("buffer should be created");
    render_component(
        &mut buffer,
        &component,
        ComponentData::Emails(vec![email("1"), email("2")]),
    );

    let first_row = first_data_row(&buffer);

    // The enter action marked the first row and the pane was re-rendered.
    loading::mark(&buffer, Anchor::Row(0));
    render_component(
        &mut buffer,
        &component,
        ComponentData::Emails(vec![email("1"), email("2")]),
    );

    let line = buffer_line(&buffer, first_row);
    assert!(
        has_spinner(&line),
        "expected a spinner in the Sel cell of the loading row, got: {line}"
    );

    // The other row keeps its empty marker.
    let other = buffer_line(&buffer, first_row + 1);
    assert!(
        !has_spinner(&other),
        "expected no spinner on the untouched row, got: {other}"
    );

    // Clearing the load restores the marker.
    loading::clear(&buffer, &Anchor::Row(0));
    let line = buffer_line(&buffer, first_row);
    assert!(
        !has_spinner(&line),
        "expected the spinner to be gone after clearing, got: {line}"
    );
}

#[nvim_oxi::test]
fn spinner_advances_in_place() {
    let component = component("Email", "List");
    let opts = OptionOpts::builder().scope(OptionScope::Local).build();
    let mut buffer = create_base_buffer(&opts).expect("buffer should be created");
    render_component(&mut buffer, &component, ComponentData::Emails(vec![email("1")]));

    let first_row = first_data_row(&buffer);

    loading::mark(&buffer, Anchor::Row(0));
    render_component(&mut buffer, &component, ComponentData::Emails(vec![email("1")]));

    let before = buffer_line(&buffer, first_row);
    let frame_before = FRAMES
        .iter()
        .find(|frame| before.contains(**frame))
        .expect("spinner should be drawn");
    let column_before = before.find(frame_before).expect("spinner column");

    loading::advance();

    let after = buffer_line(&buffer, first_row);
    let frame_after = FRAMES
        .iter()
        .find(|frame| after.contains(**frame))
        .expect("spinner should still be drawn");
    let column_after = after.find(frame_after).expect("spinner column");

    assert_ne!(
        frame_before, frame_after,
        "the frame should advance on each tick"
    );
    assert_eq!(
        column_before, column_after,
        "the spinner should turn in place"
    );
}

#[nvim_oxi::test]
fn spinner_rewrites_the_cell_instead_of_growing_the_line() {
    let component = component("Email", "List");
    let opts = OptionOpts::builder().scope(OptionScope::Local).build();
    let mut buffer = create_base_buffer(&opts).expect("buffer should be created");
    render_component(&mut buffer, &component, ComponentData::Emails(vec![email("1")]));

    let first_row = first_data_row(&buffer);

    loading::mark(&buffer, Anchor::Row(0));
    render_component(&mut buffer, &component, ComponentData::Emails(vec![email("1")]));

    let before = buffer_line(&buffer, first_row);
    let length_before = before.len();
    let frame_before = FRAMES
        .iter()
        .find(|frame| before.contains(**frame))
        .expect("spinner should be drawn");
    let column_before = before.find(frame_before).expect("spinner column");

    // Several animation ticks must rewrite the spinner cell in place: the
    // line keeps its length (only the frame character changes).
    for _ in 0..5 {
        loading::advance();
    }

    let after = buffer_line(&buffer, first_row);
    assert_eq!(
        after.len(),
        length_before,
        "the spinner must replace its cell, not insert into the line"
    );
    let frame_after = FRAMES
        .iter()
        .find(|frame| after.contains(**frame))
        .expect("spinner should still be drawn");
    let column_after = after.find(frame_after).expect("spinner column");
    assert_eq!(
        column_before, column_after,
        "the spinner should turn in place"
    );
}

#[nvim_oxi::test]
fn animator_ticks_the_spinner_in_place() {
    let component = component("Email", "List");
    let opts = OptionOpts::builder().scope(OptionScope::Local).build();
    let mut buffer = create_base_buffer(&opts).expect("buffer should be created");
    render_component(&mut buffer, &component, ComponentData::Emails(vec![email("1")]));

    let first_row = first_data_row(&buffer);

    loading::mark(&buffer, Anchor::Row(0));
    render_component(&mut buffer, &component, ComponentData::Emails(vec![email("1")]));

    let before = buffer_line(&buffer, first_row);
    let frame_before = FRAMES
        .iter()
        .find(|frame| before.contains(**frame))
        .expect("spinner should be drawn")
        .to_string();
    let column_before = before.find(frame_before.as_str()).expect("spinner column");

    // Let the animation task tick on the event loop for a while.
    api::command("call wait(400, {-> 0})").expect("wait should run");

    let after = buffer_line(&buffer, first_row);
    let frame_after = FRAMES
        .iter()
        .find(|frame| after.contains(**frame))
        .expect("spinner should still be drawn")
        .to_string();
    let column_after = after.find(frame_after.as_str()).expect("spinner column");

    assert_ne!(
        frame_before, frame_after,
        "the animation task should advance the frames over time"
    );
    assert_eq!(
        column_before, column_after,
        "the spinner should turn in place"
    );
}

#[nvim_oxi::test]
fn accounts_table_has_a_sel_column() {
    let component = component("Account", "List");
    let opts = OptionOpts::builder().scope(OptionScope::Local).build();
    let mut buffer = create_base_buffer(&opts).expect("buffer should be created");
    render_component(
        &mut buffer,
        &component,
        ComponentData::Accounts(vec![Account::new(
            "nic@aliyssium.com".into(),
            Some("imap".into()),
            true,
        )]),
    );

    // The line right after the metadata block is the header row.
    let metadata = BufferMetadata::from_buffer(&buffer, None).expect("metadata should parse");
    let header = buffer_line(&buffer, metadata.line_count + 1);
    assert!(
        header.contains("Sel"),
        "expected a Sel column on the accounts table, got: {header}"
    );
}

#[nvim_oxi::test]
fn drawer_account_node_shows_spinner_while_expanding() {
    let opts = OptionOpts::builder().scope(OptionScope::Local).build();
    let mut buffer = create_base_buffer(&opts).expect("buffer should be created");
    setup_drawer_buffer(&mut buffer).expect("drawer buffer should be set up");

    test_set_accounts(vec![Account::new("engelgasse".into(), None, true)]);

    loading::mark(&buffer, Anchor::Account("engelgasse".into()));
    render_tree(&mut buffer).expect("drawer should render");

    // The account line carries the spinner...
    let metadata = BufferMetadata::from_buffer(&buffer, None).expect("metadata should parse");
    let account_line = buffer_line(&buffer, metadata.line_count + 1);
    assert!(
        has_spinner(&account_line),
        "expected a spinner on the expanding account, got: {account_line}"
    );

    // ...until the load finishes and clears it.
    loading::clear(&buffer, &Anchor::Account("engelgasse".into()));
    let account_line = buffer_line(&buffer, metadata.line_count + 1);
    assert!(
        !has_spinner(&account_line),
        "expected the spinner to be gone after clearing, got: {account_line}"
    );

    test_set_accounts(Vec::new());
}

#[nvim_oxi::test]
fn anchor_row_restores_the_selection_marker_on_clear() {
    let component = component("Email", "List");
    let opts = OptionOpts::builder().scope(OptionScope::Local).build();
    let mut buffer = create_base_buffer(&opts).expect("buffer should be created");
    render_component(
        &mut buffer,
        &component,
        ComponentData::Emails(vec![email("1"), email("2")]),
    );

    let first_row = first_data_row(&buffer);

    // The loading row was already selected: the spinner takes its place...
    let _ = crate::utils::selection::toggle(&buffer, "1");
    loading::mark(&buffer, Anchor::Row(0));
    render_component(
        &mut buffer,
        &component,
        ComponentData::Emails(vec![email("1"), email("2")]),
    );
    let line = buffer_line(&buffer, first_row);
    assert!(has_spinner(&line), "expected the spinner to replace the marker");

    // ...and clearing the load brings the `>` marker back.
    loading::clear(&buffer, &Anchor::Row(0));
    let line = buffer_line(&buffer, first_row);
    assert!(
        line.contains('>'),
        "expected the selection marker to be restored, got: {line}"
    );

    crate::utils::selection::clear(&buffer);
}
