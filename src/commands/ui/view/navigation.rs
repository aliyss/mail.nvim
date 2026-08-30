//! Enter actions and linked-pane previews for view components.
//!
//! * [`ui_enter`] resolves the row under the cursor and runs the component's
//!   [`enter_action`](UiViewComponent::enter_action): drill down in place
//!   (`replace_view`/`expand_view`), or open the selected row in a new window
//!   to the right (`new_window`).
//! * [`pane_selection_changed`] is bound to `CursorMoved` on linked list
//!   components: debounced, it re-renders the linked pane (a reading pane)
//!   with the email under the cursor, without stealing focus.

use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use nvim_oxi::Object;
use nvim_oxi::api::opts::{OptionOpts, OptionScope};
use nvim_oxi::api::{self, Buffer, Window};
use tokio::task::AbortHandle;

use crate::api::account::Account;
use crate::api::config::Config;
use crate::api::config::ui::view::{
    UiViewComponent, UiViewComponentContextContext, UiViewEnterAction,
};
use crate::api::email::Email;
use crate::api::file::TryFile;
use crate::api::folder::Folder;
use crate::commands::UserCommand;
use crate::commands::email::get::EmailGet;
use crate::commands::email::list::EmailList;
use crate::commands::folder::list::FolderList;
use crate::commands::ui::drawer::perform_drawer_action;
use crate::commands::ui::is_drawer;
use crate::utils::buffer::metadata::BufferMetadata;
use crate::utils::buffer::render::FromBuffer;
use crate::utils::loading::{self, Anchor};
use crate::utils::render::table::context::fetch_row_id;
use crate::utils::render::{
    ASYNC_RUNTIME, ComponentData, apply_pagination, cached_pane_data, create_base_buffer,
    get_data, load_into, new_async_handle, render_into_buffer, send_async,
};

use super::{engine, instances};

/// How long the linked-pane preview waits after the cursor stops moving.
const PREVIEW_DEBOUNCE: Duration = Duration::from_millis(150);

/// Bookkeeping for the linked-pane preview debounce.
#[derive(Default)]
struct PreviewState {
    /// The `(component id, email id)` last previewed, to avoid redundant
    /// refetches while the cursor moves within the same row.
    last: Option<(String, String)>,
    /// The pending (debounced) preview task, cancelled on the next move.
    pending: Option<AbortHandle>,
}

static PREVIEW_STATE: LazyLock<Mutex<PreviewState>> =
    LazyLock::new(|| Mutex::new(PreviewState::default()));

/// The `<CR>` entry point of every list-like component. Resolves the row
/// under the cursor and runs the component's enter action. Exported to Lua as
/// `ui_enter`.
pub fn ui_enter(_: Object) {
    let buffer = api::get_current_buf();

    // The drawer is a special component with its own expand logic.
    if is_drawer(buffer.clone()) {
        perform_drawer_action();
        return;
    }

    let Ok(metadata) = BufferMetadata::from_buffer(&buffer, None) else {
        return;
    };
    let component = &metadata.component;

    let Some(target) = target_component(component, &buffer, metadata.line_count) else {
        return;
    };

    // The row under the cursor is being opened: show a spinner on it in the
    // source pane until the target has loaded. The cursor row is 1-indexed;
    // data row `i` (0-based) sits at `line_count + 3 + i` (metadata block,
    // header, separator, then the rows), so the anchor is the cursor row
    // minus `line_count + 3`.
    let row = api::get_current_win()
        .get_cursor()
        .map_or(0, |(row, _)| row)
        .saturating_sub(metadata.line_count + 3);
    let guard = loading::Guard::new(buffer.clone(), Anchor::Row(row));
    rerender_current_pane(component);

    let Ok(config) = Config::read_from_file(None) else {
        nvim_oxi::print!("failed to read config file");
        return;
    };

    match component.enter_action() {
        // For list-like components both mean "drill into the selected row in
        // the same pane"; the drawer (which expands a tree in place) is
        // handled above.
        UiViewEnterAction::ExpandView | UiViewEnterAction::ReplaceView => {
            replace_current(target, config, guard);
        }
        UiViewEnterAction::NewWindow => open_new_window(target, config, guard),
    }
}

/// Re-renders the current pane from its cached data (so a freshly marked
/// loading spinner is drawn), restoring the cursor afterwards.
fn rerender_current_pane(component: &UiViewComponent) {
    let buffer = api::get_current_buf();
    let mut window = api::get_current_win();
    let cursor = window.get_cursor().unwrap_or((1, 0));

    let Some(data) = cached_pane_data(&buffer) else {
        return;
    };

    let mut buffer = buffer;
    if let Err(err) = render_into_buffer(&mut buffer, component, data) {
        nvim_oxi::print!("failed to re-render pane: {err}");
    }

    let rows = buffer.line_count().unwrap_or(1);
    let row = cursor.0.min(rows).max(1);
    let _ = window.set_cursor(row, 0);
}

/// Resolves the component that shows the selected row's content, or `None`
/// when the row has nothing to drill into (e.g. the reading pane).
fn target_component(
    component: &UiViewComponent,
    buffer: &Buffer,
    line_count: usize,
) -> Option<UiViewComponent> {
    match component.context.command_group.as_str() {
        "Account" => {
            let account_id = fetch_row_id::<Vec<Account>>(buffer, line_count).ok()?;
            let mut target = FolderList::default_view_component()?;
            target.context.context =
                vec![UiViewComponentContextContext::AccountId(account_id)];
            Some(target)
        }
        "Folder" => {
            let account_id = component
                .context
                .get_optional_context("account_id")?
                .clone();
            let folder_id = fetch_row_id::<Vec<Folder>>(buffer, line_count).ok()?;
            let mut target = EmailList::default_view_component()?;
            target.context.context = vec![
                account_id,
                UiViewComponentContextContext::FolderId(folder_id),
            ];
            // Fetch as many emails as the pane's height fits instead of
            // falling back to the provider's default page size: the email
            // list opens in a pane as tall as the current one (splits are
            // vertical), so the current window's height fits the pane.
            apply_pagination(&mut target);
            Some(target)
        }
        "Email" if matches!(component.context.command_type.as_str(), "List" | "Thread") => {
            let account_id = component
                .context
                .get_optional_context("account_id")?
                .clone();
            let folder_id = component.context.get_optional_context("folder_id")?.clone();
            let email_id = fetch_row_id::<Vec<Email>>(buffer, line_count).ok()?;
            let mut target = EmailGet::default_view_component()?;
            target.context.context = vec![
                account_id,
                folder_id,
                UiViewComponentContextContext::EmailId(email_id),
            ];
            Some(target)
        }
        _ => None,
    }
}


/// Renders `component` into the current pane (drill down) and re-registers
/// the pane under the new component's id so links and further enter actions
/// keep resolving. `guard` clears the loading spinner of the source row when
/// the load finishes.
fn replace_current(component: UiViewComponent, config: Config, guard: loading::Guard) {
    let buffer = api::get_current_buf();
    let window = api::get_current_win();

    let old_id = BufferMetadata::from_buffer(&buffer, None)
        .ok()
        .map(|metadata| metadata.component.id)
        .unwrap_or_default();

    instances::replace(
        &old_id,
        &component.id,
        component.clone(),
        buffer.clone(),
        window,
    );
    load_into(component, config, buffer, Some(guard));
}

/// Opens `component` in a window to the right of the current one and loads it
/// asynchronously. `guard` clears the loading spinner of the source row when
/// the load finishes.
fn open_new_window(component: UiViewComponent, config: Config, guard: loading::Guard) {
    // The pane this action was triggered from: it loses width to the new
    // window and must be re-rendered at the narrower size afterwards.
    let source_buffer = api::get_current_buf();

    let window_before = api::get_current_win();
    let _ = api::command("wincmd l");
    if api::get_current_win() == window_before {
        let _ = api::command("vsplit");
    }

    let opts = OptionOpts::builder().scope(OptionScope::Local).build();
    match create_base_buffer(&opts) {
        Ok(buffer) => {
            let window = api::get_current_win();
            instances::register(&component.id, component.clone(), buffer.clone(), window);
            load_into(component, config, buffer, Some(guard));
        }
        Err(err) => nvim_oxi::print!("failed to create buffer: {err}"),
    }

    // Lay the new pane out together with the existing ones: recalculating
    // left-to-right resizes (e.g.) the email list back to its configured
    // percentage instead of leaving it at its pre-open width.
    engine::recalculate_layout(Object::nil());

    // The source pane (e.g. the email list) now shares its width with the new
    // window. Panes without an explicit layout are not resized by the layout
    // pass, so re-render it explicitly to fit the narrower pane.
    if source_buffer.is_valid() {
        engine::rerender_pane_by_buffer(&source_buffer);
    }
}

/// Binds a buffer-local `CursorMoved` autocmd that updates the pane linked to
/// `component` (see [`UiViewComponentLink`]).
///
/// # Errors
///
/// Returns an error if the autocmd cannot be created.
pub(crate) fn bind_linked_preview(buffer: &Buffer) -> anyhow::Result<()> {
    api::command(&format!(
        "autocmd CursorMoved <buffer={}> lua require('mail_nvim').pane_selection_changed()",
        buffer.handle()
    ))?;
    Ok(())
}

/// Exported to Lua as `pane_selection_changed`. Bound to `CursorMoved` on the
/// buffer of every linked list component: after a short debounce, re-renders
/// the linked pane (reading pane) with the email under the cursor.
pub fn pane_selection_changed(_: Object) {
    let buffer = api::get_current_buf();

    let Ok(metadata) = BufferMetadata::from_buffer(&buffer, None) else {
        return;
    };
    let component = &metadata.component;

    // Only list-like email components can drive a linked reading pane.
    if component.context.command_group != "Email"
        || !matches!(component.context.command_type.as_str(), "List" | "Thread")
    {
        return;
    }

    let Some(link) = &component.link else {
        return;
    };

    let Ok(email_id) = fetch_row_id::<Vec<Email>>(&buffer, metadata.line_count) else {
        return;
    };

    let Some(target) = instances::get(&link.target) else {
        return;
    };

    let selection = (component.id.clone(), email_id.clone());

    let Ok(mut state) = PREVIEW_STATE.lock() else {
        return;
    };

    // Skip refetching while the selection hasn't changed.
    if state.last.as_ref() == Some(&selection) {
        return;
    }
    state.last = Some(selection);

    // Debounce: cancel any pending preview and start a fresh one.
    if let Some(pending) = state.pending.take() {
        pending.abort();
    }

    // Fill the linked component with the selected email's context.
    let mut preview = target.component;
    preview.context.context = component
        .context
        .context
        .iter()
        .filter(|ctx| ctx.context_type() != "email_id")
        .cloned()
        .chain([UiViewComponentContextContext::EmailId(email_id)])
        .collect();

    let config = match Config::read_from_file(None) {
        Ok(config) => config,
        Err(err) => {
            nvim_oxi::print!("failed to read config file: {err}");
            return;
        }
    };

    state.pending = schedule_preview(preview, target.buffer, target.window, config);
}

/// Fetches `component`'s data after a short debounce and renders it into the
/// linked pane without stealing focus. Returns the [`AbortHandle`] of the
/// scheduled task so a newer cursor move can cancel it.
fn schedule_preview(
    component: UiViewComponent,
    buffer: Buffer,
    window: Window,
    config: Config,
) -> Option<AbortHandle> {
    let shared_component = component.clone();
    let shared_data = Arc::new(Mutex::<Option<ComponentData>>::new(None));
    let shared_data_for_async = Arc::clone(&shared_data);

    let async_handle = new_async_handle(move || {
        let mut lock = shared_data.lock().unwrap();
        if let Some(data) = lock.take() {
            let component = shared_component.clone();
            let buffer = buffer.clone();
            let window = window.clone();
            nvim_oxi::schedule(move |()| {
                render_linked_pane(&component, data, &buffer, &window);
            });
        }
    })?;

    let handle = ASYNC_RUNTIME.spawn(async move {
        tokio::time::sleep(PREVIEW_DEBOUNCE).await;
        if let Ok(data) = get_data(&component, &config).await {
            *shared_data_for_async.lock().unwrap() = Some(data);
            send_async(&async_handle);
        }
    });

    Some(handle.abort_handle())
}

/// Renders `data` into the linked pane's `window`/`buffer`, restoring the
/// user's window, buffer and cursor afterwards.
fn render_linked_pane(
    component: &UiViewComponent,
    data: ComponentData,
    buffer: &Buffer,
    window: &Window,
) {
    if !buffer.is_valid() || !window.is_valid() {
        return;
    }

    let mut orig_win = api::get_current_win();
    let orig_buf = api::get_current_buf();
    let orig_pos = orig_win.get_cursor().unwrap_or((1, 0));

    if api::set_current_win(window).is_err() {
        return;
    }

    let mut buffer = buffer.clone();
    if let Err(err) = render_into_buffer(&mut buffer, component, data) {
        nvim_oxi::print!("failed to render linked pane: {err}");
    }

    let _ = api::set_current_win(&orig_win);
    let _ = api::set_current_buf(&orig_buf);
    if orig_win.is_valid() {
        let _ = orig_win.set_cursor(orig_pos.0, orig_pos.1);
    }
}
