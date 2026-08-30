use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

use anyhow::Context;
use nvim_oxi::Object;
use nvim_oxi::api::opts::{OptionOpts, OptionScope, SetKeymapOpts};
use nvim_oxi::api::{self, Buffer};
use nvim_oxi::libuv::AsyncHandle;
use tokio::runtime::Runtime;

use crate::api::account::Account;
use crate::api::account::commands::ListAccounts;
use crate::api::config::Config;
use crate::api::config::ui::view::{UiViewComponent, UiViewComponentContextContext};
use crate::api::email::commands::{GetEmail, ListEmails, ListThreads};
use crate::api::email::{Email, EmailMessage, ThreadedEmail};
use crate::api::folder::Folder;
use crate::api::folder::commands::ListFolders;
use crate::utils::buffer::metadata::BufferMetadata;
use crate::utils::buffer::render::{FromBuffer, ToBuffer};
use crate::utils::completion;
use crate::utils::loading;
use crate::utils::render::table::context::fetch_row_id;
use crate::utils::render::table::marked::HasId;

pub mod component;
pub mod message;
pub mod pagination;
pub mod table;

pub use pagination::{apply_pagination, current_limit, current_page};

/// Creates a libuv [`AsyncHandle`] that runs `callback` on Neovim's main
/// thread, printing an error instead of panicking on failure.
///
/// Returns `None` (after reporting the error) so callers don't crash Neovim.
pub(crate) fn new_async_handle<Cb>(callback: Cb) -> Option<AsyncHandle>
where
    Cb: FnMut() + 'static,
{
    match AsyncHandle::new(callback) {
        Ok(handle) => Some(handle),
        Err(err) => {
            nvim_oxi::print!("failed to create async handle: {err}");
            None
        }
    }
}

/// Wakes the [`AsyncHandle`], printing an error instead of panicking if the
/// notification cannot be sent to Neovim.
pub(crate) fn send_async(handle: &AsyncHandle) {
    if let Err(err) = handle.send() {
        nvim_oxi::print!("failed to send async notification to Neovim: {err}");
    }
}

pub static ASYNC_RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime")
});

#[derive(Clone)]
pub enum ComponentData {
    Accounts(Vec<Account>),
    Folders(Vec<Folder>),
    Emails(Vec<Email>),
    Threads(Vec<ThreadedEmail>),
    EmailMessages(Vec<EmailMessage>),
    None,
}

/// The last [`ComponentData`] rendered into each open pane, keyed by buffer
/// handle.
///
/// The data is re-used to re-render a pane without a provider round-trip when
/// its window is resized (see
/// [`recalculate_layout`](crate::commands::ui::view::engine::recalculate_layout)):
/// the height of a message or the layout of a table depends on the pane's
/// width, so the rendered content has to be rebuilt from the structured data,
/// not from the lossy buffer text.
static PANE_DATA: LazyLock<Mutex<HashMap<i32, ComponentData>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Stores the data last rendered into `buffer`, so a later resize can rebuild
/// the pane's content from it.
pub(crate) fn cache_pane_data(buffer: &Buffer, data: &ComponentData) {
    PANE_DATA.lock().unwrap().insert(buffer.handle(), data.clone());
}

/// The width of the window each pane's content was last rendered at, keyed by
/// buffer handle.
///
/// A pane's rendered content (a table or a message) has the width it was
/// built for baked in. When the window later changes width — a split closing
/// expands the last pane, or the pane is resized directly — the pane has to
/// be re-rendered from the cached data to fill the new width. Comparing the
/// window's current width against the rendered width tells the layout pass
/// (and the resize autocommands) exactly which panes need rebuilding.
static PANE_RENDER_WIDTH: LazyLock<Mutex<HashMap<i32, u32>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Records the width `buffer`'s content was rendered at.
pub(crate) fn record_rendered_width(buffer: &Buffer, width: u32) {
    PANE_RENDER_WIDTH
        .lock()
        .unwrap()
        .insert(buffer.handle(), width);
}

/// The width `buffer`'s content was last rendered at (0 when unknown).
#[must_use]
pub(crate) fn last_rendered_width(buffer: &Buffer) -> u32 {
    PANE_RENDER_WIDTH
        .lock()
        .unwrap()
        .get(&buffer.handle())
        .copied()
        .unwrap_or(0)
}

/// Returns a copy of the data last rendered into `buffer`, if any. Stale
/// entries are dropped as buffers are closed.
#[must_use]
pub(crate) fn cached_pane_data(buffer: &Buffer) -> Option<ComponentData> {
    PANE_DATA.lock().unwrap().get(&buffer.handle()).cloned()
}

/// The 0-based data-row index of the table row the cursor is on, given the
/// metadata block's line count. A data row `i` renders at 1-indexed buffer
/// line `metadata_line_count + 3 + i` (metadata block + header + separator),
/// so the index is `cursor_row - metadata_line_count - 3`.
fn cursor_row_index(metadata_line_count: usize) -> Option<usize> {
    let (row, _) = api::get_current_win().get_cursor().ok()?;
    row.checked_sub(metadata_line_count + 3)
}

/// The id of the item under the cursor, read from the pane's cached
/// structured data (the same items that rendered the table, in the same
/// order) instead of re-parsing the rendered text — a truncated header can
/// no longer break the lookup. Returns `None` when the pane has no cached
/// list data (e.g. the drawer renders its own tree).
pub(crate) fn cached_row_id(buffer: &Buffer, metadata_line_count: usize) -> Option<String> {
    let row_index = cursor_row_index(metadata_line_count)?;
    let data = cached_pane_data(buffer)?;
    match data {
        ComponentData::Accounts(accounts) => accounts.get(row_index).map(|a| a.name().to_string()),
        ComponentData::Folders(folders) => folders.get(row_index).map(|f| f.id().to_string()),
        ComponentData::Emails(emails) => emails.get(row_index).map(|e| e.id().to_string()),
        ComponentData::Threads(emails) => emails.get(row_index).map(|e| e.id().to_string()),
        _ => None,
    }
}

/// The email under the cursor, read from the pane's cached email list or
/// thread. Returns `None` when the pane has no cached email data.
pub(crate) fn cached_row_email(buffer: &Buffer, metadata_line_count: usize) -> Option<Email> {
    let row_index = cursor_row_index(metadata_line_count)?;
    let data = cached_pane_data(buffer)?;
    match data {
        ComponentData::Emails(emails) => emails.get(row_index).cloned(),
        ComponentData::Threads(emails) => emails.get(row_index).map(|t| t.email().clone()),
        _ => None,
    }
}

fn get_optional_context_by_id<'a>(
    matcher: &str,
    component: &'a UiViewComponent,
    metadata: Option<&'a BufferMetadata>,
) -> Option<&'a UiViewComponentContextContext> {
    if let Some(ctx) = component.context.get_optional_context(matcher) {
        return Some(ctx);
    }

    if let Some(buffer_metadata) = metadata {
        return get_optional_context_by_id(matcher, &buffer_metadata.component, None);
    }

    None
}

fn get_required_context_by_id<'a>(
    matcher: &str,
    component: &'a UiViewComponent,
    metadata: Option<&'a BufferMetadata>,
) -> anyhow::Result<&'a UiViewComponentContextContext> {
    match component.context.get_required_context(matcher, None) {
        Ok(ctx) => Ok(ctx),
        Err(err) => {
            if let Some(buffer_metadata) = metadata {
                return get_required_context_by_id(matcher, &buffer_metadata.component, None);
            }
            anyhow::bail!("required context not found: {err:#}");
        }
    }
}

pub fn get_context(
    current_buffer: Option<Buffer>,
    component: &UiViewComponent,
) -> anyhow::Result<Vec<UiViewComponentContextContext>> {
    let mut context: Vec<UiViewComponentContextContext> = Vec::new();

    if component.context.command_group.as_str() == "Folder"
        && component.context.command_type == "List"
    {
        let buffer_metadata = current_buffer
            .as_ref()
            .and_then(|buf| BufferMetadata::from_buffer(buf, None).ok());

        let account_id =
            get_optional_context_by_id("account_id", component, buffer_metadata.as_ref());

        if let Some(account_id) = account_id {
            context.push(account_id.clone());
            return Ok(context);
        }

        if let Some(buffer) = current_buffer {
            let account_id = fetch_row_id::<Vec<Account>>(
                &buffer,
                buffer_metadata.map_or(0, |meta| meta.line_count),
            )
            .ok();

            let Some(account_id) = account_id else {
                return Ok(context);
            };

            context.push(UiViewComponentContextContext::AccountId(account_id));
        }
    } else if component.context.command_group.as_str() == "Email" {
        if component.context.command_type == "List" {
            let buffer_metadata = current_buffer
                .as_ref()
                .and_then(|buf| BufferMetadata::from_buffer(buf, None).ok());

            let account_id =
                get_optional_context_by_id("account_id", component, buffer_metadata.as_ref());

            let folder_id =
                get_optional_context_by_id("folder_id", component, buffer_metadata.as_ref());

            if let Some(account_id) = account_id {
                context.push(account_id.clone());
            }

            if let Some(folder_id) = folder_id {
                context.push(folder_id.clone());
            }

            if let Some(_) = folder_id
                && let Some(_) = account_id
            {
                return Ok(context);
            }

            if let Some(buffer) = current_buffer
                && let Some(buffer_metadata) = buffer_metadata
            {
                if buffer_metadata.component.context.command_group.as_str() == "Account" {
                    let Ok(account_id) =
                        fetch_row_id::<Vec<Account>>(&buffer, buffer_metadata.line_count)
                    else {
                        return Ok(context);
                    };

                    context.push(UiViewComponentContextContext::AccountId(account_id));
                } else if buffer_metadata.component.context.command_group.as_str() == "Folder" {
                    let Ok(folder_id) = fetch_row_id::<Vec<Folder>>(&buffer, buffer_metadata.line_count)
                    else {
                        return Ok(context);
                    };

                    context.push(UiViewComponentContextContext::FolderId(folder_id));
                }
            }
        } else if component.context.command_type == "Get"
            || component.context.command_type == "Thread"
        {
            let buffer_metadata = current_buffer
                .as_ref()
                .and_then(|buf| BufferMetadata::from_buffer(buf, None).ok());

            let account_id =
                get_optional_context_by_id("account_id", component, buffer_metadata.as_ref());

            let folder_id =
                get_optional_context_by_id("folder_id", component, buffer_metadata.as_ref());

            let email_id =
                get_optional_context_by_id("email_id", component, buffer_metadata.as_ref());

            if let Some(account_id) = account_id {
                context.push(account_id.clone());
            }

            if let Some(folder_id) = folder_id {
                context.push(folder_id.clone());
            }

            if let Some(email_id) = email_id {
                context.push(email_id.clone());
            }

            if let Some(_) = folder_id
                && let Some(_) = account_id
                && let Some(_) = email_id
            {
                return Ok(context);
            }

            if let Some(buffer) = current_buffer
                && let Some(buffer_metadata) = buffer_metadata
                && buffer_metadata.component.context.command_group.as_str() == "Email"
            {
                let Ok(email_id) = fetch_row_id::<Vec<Email>>(&buffer, buffer_metadata.line_count)
                else {
                    return Ok(context);
                };

                context.push(UiViewComponentContextContext::EmailId(email_id));
            }
        }
    }

    Ok(context)
}

/// Fetches `component`'s data as a child task, so a panic inside the
/// provider (e.g. a failed network call) cannot abort the whole spawn: the
/// panic is captured by the child task's [`JoinHandle`] and reported as a
/// plain failure. Callers then always notify the main thread, which clears
/// the loading spinner of the action that triggered the fetch.
pub(crate) async fn fetch_data(
    component: &UiViewComponent,
    config: &Config,
) -> Option<ComponentData> {
    let component = component.clone();
    let config = config.clone();
    let fetch = ASYNC_RUNTIME.spawn(async move { get_data(&component, &config).await });

    match fetch.await {
        Ok(Ok(data)) => Some(data),
        Ok(Err(err)) => {
            tracing::warn!("failed to fetch component data: {err:#}");
            None
        }
        Err(join_error) => {
            tracing::warn!("component data fetch panicked: {join_error}");
            None
        }
    }
}

pub async fn get_data(
    component: &UiViewComponent,
    config: &Config,
) -> anyhow::Result<ComponentData> {
    let provider = config
        .to_provider()
        .context("failed to initialize provider")?;

    match component.context.command_group.as_str() {
        "Account" => {
            if component.context.command_type == "List" {
                let accounts = provider
                    .list_accounts()
                    .context("failed to list accounts")?;
                return Ok(ComponentData::Accounts(accounts));
            }
        }
        "Folder" => {
            if component.context.command_type == "List" {
                let account_id = component.context.get_required_context("account_id", None)?;

                let folders = provider
                    .list_folders(account_id.as_str())
                    .await
                    .context("failed to list folders")?;

                // Cache the ids for live command-line completion.
                completion::cache_folders(account_id.as_str(), folders.clone());

                return Ok(ComponentData::Folders(folders));
            }
        }
        "Email" => {
            if component.context.command_type == "List" {
                let account_id = component.context.get_required_context("account_id", None)?;
                let folder_id = component.context.get_optional_context("folder_id");

                let emails = match provider
                    .list_emails(
                        account_id.as_str(),
                        folder_id.map(UiViewComponentContextContext::as_str),
                        pagination::email_list_arguments(component),
                    )
                    .await
                {
                    Ok(emails) => emails,
                    Err(_err) => {
                        anyhow::bail!("failed to list emails.");
                    }
                };

                // Cache the ids for live command-line completion.
                completion::cache_emails(
                    account_id.as_str(),
                    folder_id.map_or("", UiViewComponentContextContext::as_str),
                    emails.clone(),
                );

                return Ok(ComponentData::Emails(emails));
            } else if component.context.command_type == "Thread" {
                let account_id = component.context.get_required_context("account_id", None)?;
                let folder_id = component.context.get_optional_context("folder_id");
                let email_id = component.context.get_required_context("email_id", None)?;

                let mut emails = match provider
                    .list_threads(
                        account_id.as_str(),
                        email_id.as_str(),
                        folder_id.map(UiViewComponentContextContext::as_str),
                    )
                    .await
                {
                    Ok(emails) => emails,
                    Err(_err) => {
                        anyhow::bail!("failed to list email thread.");
                    }
                };

                let page = pagination::current_page(component);
                if let Some(limit) = pagination::current_limit(component) {
                    let start = page.saturating_sub(1) * limit;
                    emails = emails.into_iter().skip(start).take(limit).collect();
                }

                return Ok(ComponentData::Threads(emails));
            } else if component.context.command_type == "Get"
                || component.context.command_type == "Thread"
            {
                let account_id = component.context.get_required_context("account_id", None)?;
                let folder_id = component.context.get_optional_context("folder_id");
                let email_id = component.context.get_required_context("email_id", None)?;

                let emails = match provider
                    .get_emails(
                        account_id.as_str(),
                        vec![email_id.as_str()],
                        folder_id.map(UiViewComponentContextContext::as_str),
                        None,
                    )
                    .await
                {
                    Ok(emails) => emails,
                    Err(_err) => {
                        anyhow::bail!("failed to get emails.");
                    }
                };

                return Ok(ComponentData::EmailMessages(emails));
            }
        }
        _ => {}
    }

    Ok(ComponentData::None)
}

pub fn create_base_buffer(opts: &OptionOpts) -> anyhow::Result<Buffer> {
    let in_buffer_list = true;
    let is_temporary = true;
    let buffer = match api::create_buf(in_buffer_list, is_temporary) {
        Ok(buffer) => buffer,
        Err(err) => anyhow::bail!("failed to create buffer: {err}"),
    };

    if let Err(err) = api::set_current_buf(&buffer) {
        anyhow::bail!("failed to set current buffer: {err}");
    }

    let options: [(&'static str, Object); 3] = [
        // Allows users to use `ftplugin` to customize the buffer.
        ("filetype", Object::from("mail-table")),
        // Prevents users from saving the file.
        ("buftype", Object::from("nofile")),
        // Prevents users from entering INSERT mode.
        ("modifiable", Object::from(true)),
    ];

    for (name, value) in options {
        if let Err(err) = api::set_option_value(name, value, opts) {
            anyhow::bail!("failed to set option value: {err}");
        }
    }

    Ok(buffer)
}

/// Creates a new buffer and renders `data` into it according to the
/// component's type.
///
/// # Errors
///
/// Returns an error if the buffer cannot be created or written to.
pub fn render(component: &UiViewComponent, data: ComponentData) -> anyhow::Result<()> {
    let opts = OptionOpts::builder().scope(OptionScope::Local).build();
    let mut buffer = create_base_buffer(&opts)?;
    render_into_buffer(&mut buffer, component, data)
}

/// Renders a component's data into an existing `buffer`, replacing its
/// contents. Used to render components into the Mail UI drawer.
///
/// # Errors
///
/// Returns an error if the buffer cannot be written to or keymaps cannot be
/// bound.
pub fn render_into_buffer(
    buffer: &mut Buffer,
    component: &UiViewComponent,
    data: ComponentData,
) -> anyhow::Result<()> {
    // Keep the structured data around so a later window resize can rebuild
    // the pane's content without a provider round-trip.
    cache_pane_data(buffer, &data);
    // Remember the width the content was rendered at: a pane whose window
    // later changes width must be re-rendered to keep filling it.
    record_rendered_width(
        buffer,
        api::get_current_win().get_width().unwrap_or_default(),
    );

    render_buffer_content(buffer, component, |buffer, metadata| {
        component::render(component, data, buffer, metadata)
    })?;

    // Newly rendered content starts at the top of the buffer.
    let _ = api::get_current_win().set_cursor(1, 0);

    Ok(())
}

/// Clears `buffer`, writes the component's metadata block, runs
/// `write_content` and binds the common keymaps.
///
/// Returns the number of lines the metadata block occupies so callers can
/// offset their content.
///
/// # Errors
///
/// Returns an error if the buffer cannot be written to or keymaps cannot be
/// bound.
pub(crate) fn render_buffer_content(
    buffer: &mut Buffer,
    component: &UiViewComponent,
    write_content: impl FnOnce(&mut Buffer, &BufferMetadata) -> anyhow::Result<Vec<component::Keymap>>,
) -> anyhow::Result<usize> {
    if let Err(err) = api::set_current_buf(buffer) {
        anyhow::bail!("failed to set current buffer: {err}");
    }

    let opts = OptionOpts::builder().scope(OptionScope::Local).build();

    if let Err(err) = api::set_option_value("modifiable", true, &opts) {
        anyhow::bail!("failed to set option value: {err}");
    }

    // Replace the whole buffer with fresh content (metadata + component).
    if let Err(err) = buffer.set_lines(.., false, Vec::<String>::new()) {
        anyhow::bail!("failed to clear buffer content: {err}");
    }

    let metadata = match BufferMetadata::new(component.clone()).to_buffer(buffer, 0) {
        Ok(metadata) => metadata,
        Err(err) => anyhow::bail!("failed to render buffer metadata: {err}"),
    };

    let mut keymaps = component::common_keymaps();
    keymaps.extend(write_content(buffer, &metadata)?);

    let keymap_opts = SetKeymapOpts::builder().silent(true).build();

    for (mode, keys, command) in keymaps {
        if let Err(err) = buffer.set_keymap(mode, keys, &command, &keymap_opts) {
            anyhow::bail!("failed to set keymap: {err}");
        }
    }

    crate::utils::syntax::apply(buffer)?;

    // Keep the buffer editable while a loading spinner animates in it (the
    // spinner clears itself and restores `modifiable` when the load
    // finishes); otherwise lock the buffer so users cannot edit it.
    if !crate::utils::loading::is_active(buffer) {
        api::set_option_value("modifiable", false, &opts)?;
    }

    Ok(metadata.line_count)
}

/// Creates a new buffer and asynchronously loads `component`'s data into it.
///
/// `guard` is an optional loading marker (e.g. the drawer action that opened
/// the component) that is cleared when the load finishes.
pub fn load_into_new(component: UiViewComponent, config: Config, guard: Option<loading::Guard>) {
    let opts = OptionOpts::builder().scope(OptionScope::Local).build();

    let buffer = match create_base_buffer(&opts) {
        Ok(buffer) => buffer,
        Err(err) => {
            nvim_oxi::print!("failed to create buffer: {err}");
            return;
        }
    };

    load_into(component, config, buffer, guard);
}

/// Fetches the data for `component` asynchronously and renders it into
/// `buffer` once it arrives.
///
/// `guard` is an optional loading marker (e.g. the row an enter action was
/// triggered on) that is cleared when the load finishes — on success and on
/// failure alike.
pub fn load_into(
    component: UiViewComponent,
    config: Config,
    buffer: Buffer,
    guard: Option<loading::Guard>,
) {
    let mut guard = guard;
    let shared_component = Arc::new(Mutex::new(component.clone()));
    let shared_data = Arc::new(Mutex::<Option<ComponentData>>::new(None));
    let shared_data_for_async = Arc::clone(&shared_data);

    let Some(async_handle) = new_async_handle(move || {
        let data = shared_data.lock().unwrap().take();
        let guard = guard.take();
        let component_for_schedule = Arc::clone(&shared_component);
        let buffer_for_schedule = buffer.clone();
        nvim_oxi::schedule(move |()| {
            // The loading marker is cleared when the guard is dropped, no
            // matter whether the fetch succeeded.
            drop(guard);

            let Some(data) = data else {
                return;
            };
            if !buffer_for_schedule.is_valid() {
                return;
            }

            let component_info = component_for_schedule.lock().unwrap();
            let mut buffer = buffer_for_schedule;
            if let Err(err) = render_into_buffer(&mut buffer, &component_info, data) {
                nvim_oxi::print!("failed to render into buffer: {err}");
            }
        });
    }) else {
        return;
    };

    ASYNC_RUNTIME.spawn(async move {
        // Always notify the main thread so the loading marker is dropped even
        // when the fetch fails or the provider panics (the `None` data is
        // skipped).
        if let Some(data) = fetch_data(&component, &config).await {
            *shared_data_for_async.lock().unwrap() = Some(data);
        }
        send_async(&async_handle);
    });
}
