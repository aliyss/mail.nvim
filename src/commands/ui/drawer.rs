//! The Mail UI drawer.
//!
//! Modeled after [vim-himalaya-ui](https://github.com/aliyss/vim-himalaya-ui),
//! the drawer is a tree where accounts can be expanded to reveal their
//! folders, and folders can be expanded to reveal their actions (e.g. list
//! mail). Selecting an action opens the corresponding content in a window to
//! the right of the drawer.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex, RwLock};

use nvim_oxi::Object;
use nvim_oxi::api::{self, Buffer};

use crate::api::account::Account;
use crate::api::config::Config;
use crate::api::config::ui::view::{
    UiViewComponent, UiViewComponentContext, UiViewComponentContextContext, UiViewComponentType,
};
use crate::api::file::TryFile;
use crate::api::folder::Folder;
use crate::commands::UserCommand;
use crate::commands::email::list::EmailList;
use crate::utils::loading::{self, Anchor};
use crate::utils::render::{
    ASYNC_RUNTIME, ComponentData, apply_pagination, fetch_data, load_into_new, new_async_handle,
    render_buffer_content, send_async,
};

use super::{drawer_component, get_drawer_buffer};

/// The state of the Mail UI drawer, shared between the renderer and the
/// keymap callbacks.
pub(crate) static DRAWER_STATE: LazyLock<RwLock<DrawerState>> =
    LazyLock::new(|| RwLock::new(DrawerState::default()));

#[derive(Debug, Default)]
pub(crate) struct DrawerState {
    /// Accounts in the drawer, in display order.
    accounts: Vec<Account>,
    /// Folders per account (keyed by account name).
    folders: HashMap<String, Vec<Folder>>,
    /// Expansion keys: account names and `account::folder_id` pairs.
    expanded: HashSet<String>,
    /// The node layout of the last render, used to resolve actions and
    /// navigation from a cursor position.
    nodes: Vec<NodeInfo>,
}

/// A rendered line of the drawer tree.
#[derive(Debug, Clone)]
struct NodeInfo {
    /// 1-indexed buffer line of the node.
    line: usize,
    /// Index of the parent node within the nodes list.
    parent: Option<usize>,
    kind: DrawerNodeKind,
}

#[derive(Debug, Clone)]
enum DrawerNodeKind {
    Account { name: String },
    Folder { account: String, id: String },
    Action { account: String, folder_id: String },
}

fn folder_key(account: &str, id: &str) -> String {
    format!("{account}::{id}")
}

/// Builds the drawer lines and node layout from the current state.
fn build_content(state: &DrawerState) -> (Vec<String>, Vec<NodeInfo>) {
    let mut lines = Vec::new();
    let mut nodes = Vec::new();

    for account in &state.accounts {
        let name = account.name().to_string();
        let expanded = state.expanded.contains(&name);
        let icon = if expanded { "▾" } else { "▸" };
        lines.push(format!("{icon} {name}"));

        let account_index = nodes.len();
        nodes.push(NodeInfo {
            line: lines.len() - 1,
            parent: None,
            kind: DrawerNodeKind::Account { name: name.clone() },
        });

        if expanded {
            let mut folders: Vec<&Folder> = state
                .folders
                .get(&name)
                .map_or_else(Vec::new, |folders| folders.iter().collect());
            folders.sort_by(|a, b| a.id().cmp(b.id()));

            for folder in folders {
                let id = folder.id().to_string();
                let expanded = state.expanded.contains(&folder_key(&name, &id));
                let icon = if expanded { "▾" } else { "▸" };
                lines.push(format!("  {icon} {id}"));

                let folder_index = nodes.len();
                nodes.push(NodeInfo {
                    line: lines.len() - 1,
                    parent: Some(account_index),
                    kind: DrawerNodeKind::Folder {
                        account: name.clone(),
                        id: id.clone(),
                    },
                });

                if expanded {
                    lines.push("    List Mail".to_string());
                    nodes.push(NodeInfo {
                        line: lines.len() - 1,
                        parent: Some(folder_index),
                        kind: DrawerNodeKind::Action {
                            account: name.clone(),
                            folder_id: id,
                        },
                    });
                }
            }
        }
    }

    (lines, nodes)
}

/// Renders the drawer tree into `buffer`, replacing its contents.
///
/// The cursor is preserved across the re-render: toggling a node replaces the
/// buffer content, which would otherwise leave the cursor on the last line.
/// Loading nodes (see [`crate::utils::loading`]) get a spinner appended to
/// their line; its position is recorded so the animation can update it in
/// place.
///
/// # Errors
///
/// Returns an error if the buffer cannot be written to.
pub(crate) fn render_tree(buffer: &mut Buffer) -> anyhow::Result<()> {
    // Preserve the cursor row so the selection stays put across the re-render.
    let cursor_row = api::get_current_win()
        .get_cursor()
        .map_or(1, |(row, _)| row);

    let (mut lines, mut nodes) = {
        let state = DRAWER_STATE.read().unwrap();
        build_content(&state)
    };

    // Spinners to draw on the loading nodes.
    let drawer_loading: Vec<(Anchor, &'static str)> = loading::spinners(buffer)
        .into_iter()
        .filter(|(anchor, _)| !matches!(anchor, Anchor::Row(_)))
        .collect();

    let component = drawer_component();
    let mut spinner_positions: Vec<(Anchor, usize, usize)> = Vec::new();
    let line_count = render_buffer_content(buffer, &component, |buffer, metadata| {
        for (anchor, frame) in &drawer_loading {
            let Some(node) = nodes.iter().find(|node| match anchor {
                Anchor::Account(name) => matches!(
                    &node.kind,
                    DrawerNodeKind::Account { name: node_name } if node_name == name
                ),
                Anchor::Action { account, folder_id } => matches!(
                    &node.kind,
                    DrawerNodeKind::Action { account: node_account, folder_id: node_folder }
                        if node_account == account && node_folder == folder_id
                ),
                Anchor::Row(_) => false,
            }) else {
                continue;
            };

            // Append the spinner to the node's line and remember where it was
            // drawn (0-based buffer line + byte column).
            let content_line = node.line;
            lines[content_line].push(' ');
            lines[content_line].push_str(frame);
            spinner_positions.push((
                anchor.clone(),
                metadata.line_count + content_line,
                lines[content_line].len() - frame.len(),
            ));
        }

        buffer.set_lines(metadata.line_count..metadata.line_count, true, lines)?;
        Ok(Vec::new())
    })?;

    // Record the drawn positions so the animation can update (and eventually
    // remove) the spinners in place.
    for (anchor, line, column) in spinner_positions {
        loading::set_position(buffer, &anchor, line, column, String::new());
    }

    // Convert content-relative lines to 1-indexed buffer lines.
    let mut state = DRAWER_STATE.write().unwrap();
    for node in &mut nodes {
        node.line += line_count + 1;
    }
    state.nodes = nodes;
    drop(state);

    // Restore the cursor, clamped to the (possibly shorter) new content.
    let last_line = buffer.line_count()?;
    let target_row = cursor_row.min(last_line.max(1));
    let _ = api::get_current_win().set_cursor(target_row, 0);

    Ok(())
}

/// Fetches the accounts of `config` and renders the drawer tree once they
/// arrive.
pub(crate) fn load_drawer(buffer: Buffer, config: Config) {
    let component = drawer_component();

    let shared_data = Arc::new(Mutex::<Option<ComponentData>>::new(None));
    let shared_data_for_async = Arc::clone(&shared_data);

    let Some(async_handle) = new_async_handle(move || {
        let mut lock = shared_data.lock().unwrap();
        if let Some(data) = lock.take() {
            let buffer = buffer.clone();
            nvim_oxi::schedule(move |()| {
                if !buffer.is_valid() {
                    return;
                }

                let ComponentData::Accounts(accounts) = data else {
                    return;
                };

                let mut state = DRAWER_STATE.write().unwrap();
                state.accounts = accounts;
                state.folders.clear();
                state.expanded.clear();
                drop(state);

                let mut buffer = buffer;
                let _ = render_tree(&mut buffer);
            });
        }
    }) else {
        return;
    };

    ASYNC_RUNTIME.spawn(async move {
        // Always notify the main thread, even when the fetch fails or the
        // provider panics: the drawer either renders or stays as it is, but
        // nothing is left in flight.
        if let Some(data) = fetch_data(&component, &config).await {
            *shared_data_for_async.lock().unwrap() = Some(data);
        }
        send_async(&async_handle);
    });
}

fn folders_component(account: &str) -> UiViewComponent {
    UiViewComponent {
        id: "drawer-folders".into(),
        name: "Folders".into(),
        component_type: UiViewComponentType::Drawer,
        context: UiViewComponentContext {
            command_group: "Folder".into(),
            command_type: "List".into(),
            arguments: HashMap::new(),
            context: vec![UiViewComponentContextContext::AccountId(
                account.to_string(),
            )],
        },
        layout: None,
        on_enter: None,
        link: None,
    }
}

pub(crate) fn toggle_account(buffer: &Buffer, name: &str, config: Config) {
    let mut state = DRAWER_STATE.write().unwrap();

    if state.expanded.contains(name) {
        state.expanded.remove(name);
        drop(state);
        let mut buffer = buffer.clone();
        let _ = render_tree(&mut buffer);
        return;
    }

    if state.folders.contains_key(name) {
        state.expanded.insert(name.to_string());
        drop(state);
        let mut buffer = buffer.clone();
        let _ = render_tree(&mut buffer);
        return;
    }

    drop(state);

    // Folders not loaded yet: show a spinner on the account while they are
    // fetched, then expand.
    loading::mark(buffer, Anchor::Account(name.to_string()));
    let mut buffer = buffer.clone();
    let _ = render_tree(&mut buffer);

    let buffer = buffer;
    let account = name.to_string();
    let component = folders_component(&account);

    let shared_data = Arc::new(Mutex::<Option<ComponentData>>::new(None));
    let shared_data_for_async = Arc::clone(&shared_data);

    let Some(async_handle) = new_async_handle(move || {
        let data = shared_data.lock().unwrap().take();
        let buffer = buffer.clone();
        let account = account.clone();
        nvim_oxi::schedule(move |()| {
            if !buffer.is_valid() {
                return;
            }

            // The folders arrived (or the fetch failed): stop the spinner
            // before rendering the final tree.
            loading::clear(&buffer, &Anchor::Account(account.clone()));

            let Some(ComponentData::Folders(mut folders)) = data else {
                return;
            };

            folders.sort_by(|a, b| a.id().cmp(b.id()));

            let mut state = DRAWER_STATE.write().unwrap();
            state.folders.insert(account.clone(), folders);
            state.expanded.insert(account.clone());
            drop(state);

            let mut buffer = buffer;
            let _ = render_tree(&mut buffer);
        });
    }) else {
        return;
    };

    ASYNC_RUNTIME.spawn(async move {
        // Always notify the main thread so the spinner is cleared even when
        // the fetch fails or the provider panics.
        if let Some(data) = fetch_data(&component, &config).await {
            *shared_data_for_async.lock().unwrap() = Some(data);
        }
        send_async(&async_handle);
    });
}

fn toggle_folder(buffer: &Buffer, account: &str, id: &str) {
    let key = folder_key(account, id);
    let mut state = DRAWER_STATE.write().unwrap();

    if state.expanded.contains(&key) {
        state.expanded.remove(&key);
    } else {
        state.expanded.insert(key);
    }

    drop(state);
    let mut buffer = buffer.clone();
    let _ = render_tree(&mut buffer);
}

pub(crate) fn open_mail_list(account: &str, folder_id: &str, config: Config) {
    let Some(mut component) = EmailList::default_view_component() else {
        return;
    };
    component.context.context = vec![
        UiViewComponentContextContext::AccountId(account.to_string()),
        UiViewComponentContextContext::FolderId(folder_id.to_string()),
    ];

    apply_pagination(&mut component);

    // Show a spinner on the action while the mail list loads in the pane
    // that opens to the right of the drawer.
    let guard = get_drawer_buffer().map(|buffer| {
        let guard = loading::Guard::new(
            buffer.clone(),
            Anchor::Action {
                account: account.to_string(),
                folder_id: folder_id.to_string(),
            },
        );
        let mut buffer = buffer;
        let _ = render_tree(&mut buffer);
        guard
    });

    // Open the content in a window to the right of the drawer.
    let window_before = api::get_current_win();
    let _ = api::command("wincmd l");
    if api::get_current_win() == window_before {
        let _ = api::command("vsplit");
    }

    load_into_new(component, config, guard);
}

fn node_at_cursor() -> Option<NodeInfo> {
    let row = api::get_current_win().get_cursor().ok()?.0;

    let state = DRAWER_STATE.read().unwrap();
    state.nodes.iter().find(|node| node.line == row).cloned()
}

/// Toggles the node under the cursor: accounts and folders expand/collapse,
/// actions open their content. Exported to Lua as `drawer_action`.
pub fn drawer_action(_: Object) {
    perform_drawer_action();
}

/// The drawer's enter action: toggles the node under the cursor. Accounts and
/// folders expand/collapse in place, actions open their content in a window
/// to the right.
pub(crate) fn perform_drawer_action() {
    let Some(buffer) = get_drawer_buffer() else {
        return;
    };
    let Some(node) = node_at_cursor() else {
        return;
    };

    let config = match Config::read_from_file(None) {
        Ok(config) => config,
        Err(err) => {
            nvim_oxi::print!("failed to read config file: {err}");
            return;
        }
    };

    match node.kind {
        DrawerNodeKind::Account { name } => toggle_account(&buffer, &name, config),
        DrawerNodeKind::Folder { account, id } => toggle_folder(&buffer, &account, &id),
        DrawerNodeKind::Action { account, folder_id } => open_mail_list(&account, &folder_id, config),
    }
}

/// Replaces the drawer's accounts with `accounts` (used by tests).
pub(crate) fn test_set_accounts(accounts: Vec<Account>) {
    let mut state = DRAWER_STATE.write().unwrap();
    state.accounts = accounts;
    state.folders.clear();
    state.expanded.clear();
}

/// Moves the cursor to the next/previous sibling of the node under the
/// cursor. Exported to Lua as `drawer_goto_sibling`.
///
/// # Panics
///
/// Panics if the drawer state lock is poisoned.
pub fn drawer_goto_sibling(arg: Object) {
    let delta: i64 = arg.try_into().unwrap_or_default();

    if get_drawer_buffer().is_none() {
        return;
    }
    let Some(current) = node_at_cursor() else {
        return;
    };

    let state = DRAWER_STATE.read().unwrap();
    let Some(current_index) = state
        .nodes
        .iter()
        .position(|node| node.line == current.line)
    else {
        return;
    };
    let parent = state.nodes[current_index].parent;

    let siblings: Vec<usize> = state
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| node.parent == parent)
        .map(|(index, _)| index)
        .collect();

    let Some(position) = siblings.iter().position(|&index| index == current_index) else {
        return;
    };

    let next = if delta > 0 {
        position
            .saturating_add(usize::try_from(delta).unwrap_or(usize::MAX))
            .min(siblings.len() - 1)
    } else {
        position.saturating_sub(usize::try_from(delta.unsigned_abs()).unwrap_or(usize::MAX))
    };
    let target_line = state.nodes[siblings[next]].line;
    drop(state);

    let _ = api::get_current_win().set_cursor(target_line, 0);
}

/// Moves the cursor to the first child or the parent of the node under the
/// cursor. Exported to Lua as `drawer_goto_node`.
///
/// # Panics
///
/// Panics if the drawer state lock is poisoned.
pub fn drawer_goto_node(arg: Object) {
    let delta: i64 = arg.try_into().unwrap_or_default();

    if get_drawer_buffer().is_none() {
        return;
    }
    let Some(current) = node_at_cursor() else {
        return;
    };

    let state = DRAWER_STATE.read().unwrap();
    let Some(current_index) = state
        .nodes
        .iter()
        .position(|node| node.line == current.line)
    else {
        return;
    };

    let target_line = if delta > 0 {
        state
            .nodes
            .iter()
            .find(|node| node.parent == Some(current_index))
            .map(|node| node.line)
    } else {
        state.nodes[current_index]
            .parent
            .map(|parent| state.nodes[parent].line)
    };
    drop(state);

    if let Some(line) = target_line {
        let _ = api::get_current_win().set_cursor(line, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> DrawerState {
        DrawerState {
            accounts: vec![Account::new("engelgasse".into(), None, true)],
            folders: HashMap::from([(
                "engelgasse".to_string(),
                vec![
                    Folder::new("INBOX.Sent".into(), None, None, None, false),
                    Folder::new("INBOX".into(), None, None, None, true),
                ],
            )]),
            expanded: HashSet::from(["engelgasse".to_string()]),
            nodes: Vec::new(),
        }
    }

    #[test]
    fn collapsed_account_renders_single_line() {
        let state = DrawerState {
            expanded: HashSet::new(),
            ..state()
        };

        let (lines, nodes) = build_content(&state);
        assert_eq!(lines, vec!["▸ engelgasse".to_string()]);
        assert_eq!(nodes.len(), 1);
        assert!(matches!(nodes[0].kind, DrawerNodeKind::Account { .. }));
    }

    #[test]
    fn expanded_account_renders_folders_sorted() {
        let (lines, nodes) = build_content(&state());
        assert_eq!(
            lines,
            vec![
                "▾ engelgasse".to_string(),
                "  ▸ INBOX".to_string(),
                "  ▸ INBOX.Sent".to_string(),
            ]
        );
        assert_eq!(nodes.len(), 3);
        assert_eq!(nodes[1].parent, Some(0));
        assert_eq!(nodes[2].parent, Some(0));
    }

    #[test]
    fn expanded_folder_renders_actions() {
        let mut state = state();
        state.expanded.insert("engelgasse::INBOX.Sent".to_string());

        let (lines, nodes) = build_content(&state);
        assert_eq!(
            lines,
            vec![
                "▾ engelgasse".to_string(),
                "  ▸ INBOX".to_string(),
                "  ▾ INBOX.Sent".to_string(),
                "    List Mail".to_string(),
            ]
        );
        assert_eq!(nodes.len(), 4);
        assert_eq!(nodes[3].parent, Some(2));
        assert!(matches!(
            nodes[3].kind,
            DrawerNodeKind::Action { ref account, ref folder_id }
                if account == "engelgasse" && folder_id == "INBOX.Sent"
        ));
    }
}
