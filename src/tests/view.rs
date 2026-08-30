//! Integration tests for the file-defined view engine, run inside a real
//! (headless) Neovim via `#[nvim_oxi::test]`.

use std::collections::HashMap;

use nvim_oxi::Object;
use nvim_oxi::api::opts::{GetAutocmdsOpts, OptionOpts, OptionScope};
use nvim_oxi::api::{self};

use crate::api::account::Account;
use crate::api::config::ui::view::{
    UiView, UiViewComponent, UiViewComponentContext, UiViewComponentLayout, UiViewComponentLink,
    UiViewComponentType,
};
use crate::commands::ui::view::engine::{create_view_layout, recalculate_layout};
use crate::commands::ui::view::instances;
use crate::utils::render::{ComponentData, create_base_buffer, render_into_buffer};

fn component(
    id: &str,
    component_type: UiViewComponentType,
    layout: Option<UiViewComponentLayout>,
) -> UiViewComponent {
    UiViewComponent {
        id: id.into(),
        name: id.into(),
        component_type,
        context: UiViewComponentContext {
            command_group: "Email".into(),
            command_type: "List".into(),
            arguments: HashMap::new(),
            context: Vec::new(),
        },
        layout,
        on_enter: None,
        link: None,
    }
}

fn layout(width: u32, size_as_percentage: bool) -> UiViewComponentLayout {
    UiViewComponentLayout {
        position: "left".into(),
        content_scrollable: (true, true),
        location: (0, 0),
        size: (width, None),
        size_as_percentage,
    }
}

fn window_info() -> Vec<(usize, u32, String)> {
    let mut windows: Vec<(usize, u32, String)> = api::list_wins()
        .filter_map(|window| {
            let (_, col) = window.get_position().ok()?;
            let width = window.get_width().ok()?;
            let buffer = window.get_buf().ok()?;

            let opts = OptionOpts::builder()
                .scope(OptionScope::Local)
                .buf(buffer.clone())
                .build();
            let filetype = api::get_option_value::<String>("filetype", &opts).unwrap_or_default();

            Some((col, width, filetype))
        })
        .collect();
    windows.sort_by_key(|(col, _, _)| *col);
    windows
}

#[nvim_oxi::test]
fn percentage_layout_creates_three_panes() {
    let total = api::get_current_win().get_width().unwrap_or_default();

    let view = UiView {
        name: "outlook".into(),
        components: vec![
            component(
                "drawer",
                UiViewComponentType::Drawer,
                Some(layout(30, true)),
            ),
            component("list", UiViewComponentType::List, Some(layout(50, true))),
            component("reading", UiViewComponentType::Content, None),
        ],
    };

    let buffers = create_view_layout(view).expect("expected layout to be created");
    assert_eq!(buffers.len(), 3, "expected one buffer per component");

    let windows = window_info();
    assert_eq!(
        windows.len(),
        3,
        "expected three windows in a fresh session"
    );

    let widths: Vec<u32> = windows.iter().map(|(_, width, _)| *width).collect();
    assert_eq!(widths.len(), 3);

    // The panes must be sized left-to-right: drawer ~30%, list ~50% of the
    // remaining space, reading pane gets whatever is left.
    let expected_first = max(1, expected_percent(total, 30));
    let expected_second = max(1, total.saturating_sub(expected_first) * 50 / 100);
    assert!(
        widths[0].abs_diff(expected_first) <= 2,
        "drawer pane width {} differs too much from {expected_first}",
        widths[0]
    );
    assert!(
        widths[1].abs_diff(expected_second) <= 2,
        "list pane width {} differs too much from {expected_second}",
        widths[1]
    );

    let sum: u32 = widths.iter().sum();
    assert!(
        sum.abs_diff(total) <= 2,
        "pane widths {widths:?} should cover the {total} column screen"
    );

    // The drawer pane is a drawer, the rest are mail-table buffers.
    assert_eq!(windows[0].2, "mail-ui");
    assert_eq!(windows[1].2, "mail-table");
    assert_eq!(windows[2].2, "mail-table");
}

#[nvim_oxi::test]
fn empty_view_creates_no_windows() {
    let view = UiView {
        name: "empty".into(),
        components: Vec::new(),
    };
    let buffers = create_view_layout(view).expect("expected empty layout to succeed");
    assert!(buffers.is_empty());

    let windows = window_info();
    assert_eq!(
        windows.len(),
        1,
        "expected only the original window to remain"
    );
}

#[nvim_oxi::test]
fn drawer_pane_is_not_modifiable() {
    let view = UiView {
        name: "drawer".into(),
        components: vec![component(
            "drawer",
            UiViewComponentType::Drawer,
            Some(layout(25, true)),
        )],
    };

    let buffers = create_view_layout(view).expect("expected layout to be created");
    let (_, buffer) = &buffers[0];

    let opts = OptionOpts::builder()
        .scope(OptionScope::Local)
        .buf(buffer.clone())
        .build();

    let modifiable: bool = api::get_option_value("modifiable", &opts).unwrap_or_default();
    assert!(!modifiable, "drawer panes should not be modifiable");
}

#[nvim_oxi::test]
fn fixed_width_pane_respects_columns() {
    let total = api::get_current_win().get_width().unwrap_or_default();

    let view = UiView {
        name: "fixed".into(),
        components: vec![
            component(
                "drawer",
                UiViewComponentType::Drawer,
                Some(layout(30, false)),
            ),
            component("list", UiViewComponentType::List, None),
        ],
    };

    let _ = create_view_layout(view).expect("expected layout to be created");
    let windows = window_info();
    assert_eq!(windows.len(), 2);

    // A fixed 30-column drawer.
    assert!(
        windows[0].1.abs_diff(30) <= 2,
        "drawer pane width {} differs too much from 30",
        windows[0].1
    );

    // The list pane covers the rest.
    let sum: u32 = windows.iter().map(|(_, width, _)| *width).sum();
    assert!(sum.abs_diff(total) <= 2);
}

/// An outlook-style view: a 30% drawer, a 50% email list and a reading pane
/// that fills whatever is left.
fn outlook_view() -> UiView {
    UiView {
        name: "outlook".into(),
        components: vec![
            component(
                "drawer",
                UiViewComponentType::Drawer,
                Some(layout(30, true)),
            ),
            component("list", UiViewComponentType::List, Some(layout(50, true))),
            component("reading", UiViewComponentType::Content, None),
        ],
    }
}

#[allow(clippy::cast_possible_truncation)] // percentages are ≤ 100, result fits.
fn expected_percent(total: u32, percent: u32) -> u32 {
    (u64::from(percent) * u64::from(total) / 100) as u32
}

fn max(a: u32, b: u32) -> u32 {
    a.max(b)
}

fn cursor_moved_commands(buffer: &nvim_oxi::api::Buffer) -> Vec<String> {
    // The filtered opts of `get_autocmds` are not serialized correctly by the
    // vendored nvim-oxi, so query everything and filter in Rust.
    let opts = GetAutocmdsOpts::builder().build();

    api::get_autocmds(&opts)
        .map(|infos| {
            infos
                .filter(|info| info.event == "CursorMoved" && info.buffer.as_ref() == Some(buffer))
                .map(|info| info.command)
                .collect()
        })
        .unwrap_or_default()
}

#[nvim_oxi::test]
fn linked_list_registers_instance_and_cursor_moved_autocmd() {
    let mut list = component("emails", UiViewComponentType::List, Some(layout(50, true)));
    list.link = Some(UiViewComponentLink {
        target: "reading".into(),
    });

    let view = UiView {
        name: "outlook".into(),
        components: vec![
            list,
            component("reading", UiViewComponentType::Content, None),
        ],
    };

    let buffers = create_view_layout(view).expect("expected layout to be created");
    assert_eq!(buffers.len(), 2);

    // Every pane is registered under its component id.
    let list_instance = instances::get("emails").expect("list pane should be registered");
    let reading_instance = instances::get("reading").expect("reading pane should be registered");
    assert_eq!(list_instance.component.id, "emails");
    assert_eq!(reading_instance.component.id, "reading");

    // The linked list gets a CursorMoved autocmd driving the preview...
    let commands = cursor_moved_commands(&list_instance.buffer);
    assert!(
        commands
            .iter()
            .any(|command| command.contains("pane_selection_changed")),
        "expected a preview autocmd on the linked list, got {commands:?}"
    );

    // ...but the reading pane itself does not.
    let commands = cursor_moved_commands(&reading_instance.buffer);
    assert!(
        !commands
            .iter()
            .any(|command| command.contains("pane_selection_changed")),
        "expected no preview autocmd on the reading pane, got {commands:?}"
    );

    instances::clear();
}

#[nvim_oxi::test]
fn unlinked_components_have_no_preview_autocmd() {
    let view = UiView {
        name: "plain".into(),
        components: vec![component(
            "list",
            UiViewComponentType::List,
            Some(layout(50, true)),
        )],
    };

    let buffers = create_view_layout(view).expect("expected layout to be created");
    let (_, buffer) = &buffers[0];

    let commands = cursor_moved_commands(buffer);
    assert!(
        !commands
            .iter()
            .any(|command| command.contains("pane_selection_changed")),
        "expected no preview autocmd without a link, got {commands:?}"
    );

    instances::clear();
}

#[nvim_oxi::test]
fn recalculate_layout_reapplies_the_configured_widths() {
    let total = api::get_current_win().get_width().unwrap_or_default();

    let _ = create_view_layout(outlook_view()).expect("expected layout to be created");
    assert_eq!(instances::all().len(), 3);

    // A window changed: drag the pane boundary so the drawer takes most of
    // the screen.
    let mut drawer_window = instances::get("drawer").expect("drawer pane").window;
    drawer_window
        .set_width(2 * total / 3)
        .expect("width should set");

    recalculate_layout(Object::nil());

    let windows = window_info();
    assert_eq!(windows.len(), 3, "pane count must not change");

    // The layout is recomputed from the components' layouts: drawer ~30%,
    // list ~50% of the remaining space, reading pane the rest.
    let expected_first = max(1, expected_percent(total, 30));
    let expected_second = max(1, total.saturating_sub(expected_first) * 50 / 100);
    assert!(
        windows[0].1.abs_diff(expected_first) <= 2,
        "drawer pane width {} differs too much from {expected_first}",
        windows[0].1
    );
    assert!(
        windows[1].1.abs_diff(expected_second) <= 2,
        "list pane width {} differs too much from {expected_second}",
        windows[1].1
    );
    let sum: u32 = windows.iter().map(|(_, width, _)| *width).sum();
    assert!(
        sum.abs_diff(total) <= 2,
        "pane widths {windows:?} should cover the {total} column screen"
    );

    instances::clear();
}

#[nvim_oxi::test]
fn recalculate_layout_resizes_when_a_mail_window_is_added() {
    let total = api::get_current_win().get_width().unwrap_or_default();

    // A two-pane view: the list is the rightmost pane, so it fills whatever
    // the drawer leaves.
    let view = UiView {
        name: "outlook".into(),
        components: vec![
            component("drawer", UiViewComponentType::Drawer, Some(layout(30, true))),
            component("list", UiViewComponentType::List, Some(layout(50, true))),
        ],
    };
    let _ = create_view_layout(view).expect("expected layout to be created");
    let list_window = instances::get("list").expect("list pane").window;
    let list_before = list_window.get_width().expect("list width");
    let _ = api::set_current_win(&list_window);

    // Open a mail the way `open_new_window` does: move to the rightmost
    // window, split, and register the new pane.
    let window_before = api::get_current_win();
    let _ = api::command("wincmd l");
    if api::get_current_win() == window_before {
        let _ = api::command("vsplit");
    }
    let opts = OptionOpts::builder().scope(OptionScope::Local).build();
    let buffer = create_base_buffer(&opts).expect("expected a base buffer");
    let window = api::get_current_win();
    instances::register(
        "mail",
        component("mail", UiViewComponentType::File, None),
        buffer,
        window,
    );

    recalculate_layout(Object::nil());

    let windows = window_info();
    assert_eq!(windows.len(), 3, "expected the mail pane to join the view");

    // The new pane is the rightmost one and fills the remaining space, and
    // the list is resized back to its configured share of the screen.
    let expected_first = max(1, expected_percent(total, 30));
    let expected_second = max(1, total.saturating_sub(expected_first) * 50 / 100);
    assert!(
        windows[0].1.abs_diff(expected_first) <= 2,
        "drawer pane width {} differs too much from {expected_first}",
        windows[0].1
    );
    assert!(
        windows[1].1.abs_diff(expected_second) <= 2,
        "list pane width {} differs too much from {expected_second}",
        windows[1].1
    );
    assert!(
        list_before.abs_diff(windows[1].1) > 2,
        "expected the list to resize when the mail opens (was {list_before}, now {})",
        windows[1].1
    );
    let sum: u32 = windows.iter().map(|(_, width, _)| *width).sum();
    assert!(
        sum.abs_diff(total) <= 2,
        "pane widths {windows:?} should cover the {total} column screen"
    );

    instances::clear();
}

#[nvim_oxi::test]
fn recalculate_layout_rebuilds_changed_panes_from_the_cached_data() {
    // A wide grid leaves the account names readable even after the panes are
    // re-laid out (the list is 50% of the space the drawer leaves).
    api::command("set columns=200").expect("columns should resize");

    let view = UiView {
        name: "outlook".into(),
        components: vec![
            component("drawer", UiViewComponentType::Drawer, Some(layout(30, true))),
            component("list", UiViewComponentType::List, Some(layout(50, true))),
            component("reading", UiViewComponentType::Content, None),
        ],
    };

    let _ = create_view_layout(view).expect("expected layout to be created");

    // Render a table into the list pane so there is cached data to rebuild
    // the content from.
    let list_instance = instances::get("list").expect("list pane");
    let mut list_buffer = list_instance.buffer.clone();
    render_into_buffer(
        &mut list_buffer,
        &list_instance.component,
        ComponentData::Accounts(vec![
            Account::new("nic@aliyssium.com".into(), Some("imap".into()), true),
            Account::new("bob@example.com".into(), None, false),
        ]),
    )
    .expect("expected the list to render");

    // Resize the drawer so the list pane width changes too.
    let mut drawer_window = instances::get("drawer").expect("drawer pane").window;
    drawer_window.set_width(40).expect("width should set");

    recalculate_layout(Object::nil());

    // The changed pane was rebuilt from the cache, not wiped.
    let list_buffer = list_instance.buffer;
    let lines: String = list_buffer
        .get_lines(.., true)
        .expect("lines should be readable")
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        lines.contains("bob@example.com"),
        "expected the list content to survive the re-layout, got:\n{lines}"
    );

    instances::clear();
}
