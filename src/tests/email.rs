//! Integration tests for multi-select in email lists, run inside a real
//! (headless) Neovim via `#[nvim_oxi::test]`.

use std::collections::{HashMap, HashSet};

use chrono::Utc;
use nvim_oxi::Object;
use nvim_oxi::api::opts::{GetExtmarksOpts, OptionOpts, OptionScope, SetMarkOpts};
use nvim_oxi::api::types::{ExtmarkPosition, GetExtmarksNamespaceId, OneOrMore};
use nvim_oxi::api::{self, Buffer};

use crate::api::config::ui::view::{
    UiViewComponent, UiViewComponentContext, UiViewComponentContextContext, UiViewComponentType,
};
use crate::api::email::{Email, EmailFlag, Mailbox};
use crate::commands::email::manage::resolve_email_context;
use crate::commands::email::selection::{
    email_clear_selection, email_select_visual_range, email_toggle_selection,
};
use crate::utils::buffer::metadata::BufferMetadata;
use crate::utils::buffer::render::FromBuffer;
use crate::utils::render::table::render::Table;
use crate::utils::render::{ComponentData, create_base_buffer, render_into_buffer};
use crate::utils::selection;

fn email(id: &str) -> Email {
    email_with(id, &[], false)
}

fn email_with(id: &str, flags: &[EmailFlag], has_attachment: bool) -> Email {
    Email::new(
        id.to_string(),
        flags.iter().cloned().collect(),
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
        has_attachment,
    )
}

fn email_list_component() -> UiViewComponent {
    UiViewComponent {
        id: "test-emails".into(),
        name: "Emails".into(),
        component_type: UiViewComponentType::List,
        context: UiViewComponentContext {
            command_group: "Email".into(),
            command_type: "List".into(),
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

/// Renders a two-row email list into a fresh buffer and returns it with the
/// 1-indexed line of the first data row.
fn render_email_list() -> (Buffer, usize) {
    let opts = OptionOpts::builder().scope(OptionScope::Local).build();
    let mut buffer = create_base_buffer(&opts).expect("buffer should be created");
    render_into_buffer(
        &mut buffer,
        &email_list_component(),
        ComponentData::Emails(vec![email("1"), email("2")]),
    )
    .expect("email list should render");

    let metadata = BufferMetadata::from_buffer(&buffer, None).expect("metadata should parse");
    let table =
        Table::<Vec<Email>>::from_buffer(&buffer, Some(metadata.line_count)).expect("table parses");
    let first_row_line = table.offset + 1;

    let _ = api::get_current_win().set_cursor(first_row_line, 0);
    (buffer, first_row_line)
}

fn buffer_line(buffer: &Buffer, line: usize) -> String {
    buffer
        .get_lines(line - 1..line, true)
        .expect("line should be readable")
        .map(|s| s.to_string())
        .next()
        .unwrap_or_default()
}

#[nvim_oxi::test]
fn space_toggles_the_row_under_the_cursor_and_steps_down() {
    let (buffer, first_row_line) = render_email_list();

    email_toggle_selection(Object::nil());

    // The email is selected and its row now shows the marker.
    assert_eq!(
        selection::selected_ids(&buffer),
        HashSet::from(["1".to_string()])
    );
    assert!(
        buffer_line(&buffer, first_row_line).contains('>'),
        "expected the selected row to be marked"
    );

    // The cursor stepped down so `<Space>` can keep selecting.
    let (row, _) = api::get_current_win()
        .get_cursor()
        .expect("cursor position");
    assert_eq!(row, first_row_line + 1);

    // Actions resolve to the selection (not just the cursor row).
    let context = resolve_email_context().expect("context should resolve");
    assert_eq!(context.email_ids, vec!["1".to_string()]);

    // Selecting the second row too makes actions target both.
    email_toggle_selection(Object::nil());
    assert_eq!(
        selection::selected_ids(&buffer),
        HashSet::from(["1".to_string(), "2".to_string()])
    );

    let context = resolve_email_context().expect("context should resolve");
    assert_eq!(context.email_ids.len(), 2);

    // `u` clears the selection and drops the markers.
    email_clear_selection(Object::nil());
    assert!(selection::selected_ids(&buffer).is_empty());
    assert!(
        !buffer_line(&buffer, first_row_line).contains('>'),
        "expected the marker to be gone after clearing"
    );
}

#[nvim_oxi::test]
fn visual_selection_marks_every_row_of_the_range() {
    let (mut buffer, first_row_line) = render_email_list();
    let mark_opts = SetMarkOpts::builder().build();

    // Simulate a linewise visual selection spanning both data rows.
    buffer
        .set_mark('<', first_row_line, 0, &mark_opts)
        .expect("start mark should set");
    buffer
        .set_mark('>', first_row_line + 1, 0, &mark_opts)
        .expect("end mark should set");

    email_select_visual_range(Object::nil());

    // Every row of the visual range is marked as selected.
    assert_eq!(
        selection::selected_ids(&buffer),
        HashSet::from(["1".to_string(), "2".to_string()])
    );

    // Actions now resolve to both emails.
    let context = resolve_email_context().expect("context should resolve");
    assert_eq!(context.email_ids.len(), 2);

    // And the marker column was re-rendered in the buffer.
    assert!(
        buffer_line(&buffer, first_row_line).contains('>'),
        "expected the first row to be marked"
    );
    assert!(
        buffer_line(&buffer, first_row_line + 1).contains('>'),
        "expected the second row to be marked"
    );

    selection::clear(&buffer);
}

/// The highlight groups applied to `buffer`'s rendered table, read back from
/// its extmarks.
fn highlight_groups(buffer: &Buffer) -> Vec<String> {
    let opts = GetExtmarksOpts::builder().details(true).build();
    let extmarks: Vec<_> = buffer
        .get_extmarks(
            GetExtmarksNamespaceId::All,
            ExtmarkPosition::ByTuple((0, 0)),
            ExtmarkPosition::ByTuple((usize::MAX, usize::MAX)),
            &opts,
        )
        .expect("extmarks should be readable")
        .collect();

    extmarks
        .iter()
        .filter_map(|(_, _, _, info)| {
            info.as_ref().and_then(|i| i.hl_group.as_ref()).map(|group| match group {
                OneOrMore::One(group) => group.clone(),
                OneOrMore::List(groups) => groups.join(","),
            })
        })
        .collect()
}

#[nvim_oxi::test]
fn email_rows_are_color_coded_by_flags() {
    let (buffer, _) = render_email_list();

    // The table renderer tags the header row and the unread subjects with
    // highlight groups (defined in `mail-table.vim`).
    let groups = highlight_groups(&buffer);

    assert!(
        groups.iter().any(|group| group == "MailTableHeader"),
        "expected the header to be color coded, got: {groups:?}"
    );
    assert!(
        groups.iter().any(|group| group == "MailTableUnread"),
        "expected unread subjects to be color coded, got: {groups:?}"
    );
}

#[nvim_oxi::test]
fn answered_attachment_and_selection_are_color_coded() {
    let opts = OptionOpts::builder().scope(OptionScope::Local).build();
    let mut buffer = create_base_buffer(&opts).expect("buffer should be created");
    let component = email_list_component();
    let data = ComponentData::Emails(vec![
        email_with("1", &[EmailFlag::Answered], true),
        email_with("2", &[EmailFlag::Flagged], false),
    ]);

    render_into_buffer(&mut buffer, &component, data.clone()).expect("render should succeed");

    // Select the answered email: its `Sel` marker turns cyan, its subject
    // green and its attachment cell magenta.
    let _ = selection::toggle(&buffer, "1");
    render_into_buffer(&mut buffer, &component, data).expect("re-render should succeed");

    let groups = highlight_groups(&buffer);
    assert!(
        groups.iter().any(|group| group == "MailTableAnswered"),
        "expected answered subjects to be color coded, got: {groups:?}"
    );
    assert!(
        groups.iter().any(|group| group == "MailTableAttachment"),
        "expected attachment cells to be color coded, got: {groups:?}"
    );
    assert!(
        groups.iter().any(|group| group == "MailTableSelected"),
        "expected the selection marker to be color coded, got: {groups:?}"
    );

    selection::clear(&buffer);
}

#[nvim_oxi::test]
fn actions_fall_back_to_the_cursor_email_without_selection() {
    let (buffer, first_row_line) = render_email_list();

    // Move to the second row: with no selection, actions target that email.
    let _ = api::get_current_win().set_cursor(first_row_line + 1, 0);
    let context = resolve_email_context().expect("context should resolve");
    assert_eq!(context.email_ids, vec!["2".to_string()]);

    // The marker survives re-renders with an existing selection (e.g.
    // pagination).
    let _ = selection::toggle(&buffer, "2");
    let mut buffer = buffer;
    render_into_buffer(
        &mut buffer,
        &email_list_component(),
        ComponentData::Emails(vec![email("1"), email("2")]),
    )
    .expect("email list should re-render");
    let metadata = BufferMetadata::from_buffer(&buffer, None).expect("metadata should parse");
    let table =
        Table::<Vec<Email>>::from_buffer(&buffer, Some(metadata.line_count)).expect("table parses");
    assert!(
        buffer_line(&buffer, table.offset + 2).contains('>'),
        "expected the re-render to keep the selection marker"
    );
}
