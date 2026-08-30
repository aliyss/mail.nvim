//! Email thread commands.
//!
//! `MailEmailThread` renders the thread of the email under the cursor,
//! `MailEmailThreadList` does the same but paginated, and
//! `MailEmailThreadNext`/`MailEmailThreadPrevious` open the neighbouring
//! email of the current one inside its thread.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::api::config::Config;
use crate::api::config::ui::view::{
    UiViewComponent, UiViewComponentContext, UiViewComponentContextContext, UiViewComponentType,
};
use crate::api::email::commands::ListThreads;
use crate::api::file::TryFile;
use crate::commands::email::manage::resolve_cursor_email_context;
use crate::commands::prelude::*;
use crate::utils::render::pagination::apply_pagination;
use crate::utils::render::{
    ASYNC_RUNTIME, get_context, load_into, load_into_new, new_async_handle, send_async,
};

/// The [`UiViewComponent`] the thread commands share.
fn thread_component() -> UiViewComponent {
    UiViewComponent {
        id: "command-envelope-thread".into(),
        name: "EmailThread".into(),
        component_type: UiViewComponentType::Table,
        context: UiViewComponentContext {
            command_group: "Email".into(),
            command_type: "Thread".into(),
            arguments: HashMap::new(),
            context: Vec::new(),
        },
        layout: None,
        on_enter: None,
        link: None,
    }
}

/// Resolves the account, folder and email to show the thread for, then loads
/// the thread component into a new buffer (optionally paginated).
fn open_thread(paginated: bool) {
    let config = match Config::read_from_file(None) {
        Ok(config) => config,
        Err(err) => {
            nvim_oxi::print!("failed to read config: {err}");
            return;
        }
    };

    let current_buffer = api::get_current_buf();

    let mut view_component = thread_component();

    let context = match get_context(Some(current_buffer), &view_component) {
        Ok(context) => context,
        Err(err) => {
            nvim_oxi::print!("failed to get context: {err}");
            return;
        }
    };

    view_component.context.context = context;

    if paginated {
        apply_pagination(&mut view_component);
    }

    load_into_new(view_component, config, None);
}

/// A [`UiViewComponent`] displaying a single email message.
fn email_message_component() -> UiViewComponent {
    UiViewComponent {
        id: "thread-navigation".into(),
        name: "ThreadEmail".into(),
        component_type: UiViewComponentType::File,
        context: UiViewComponentContext {
            command_group: "Email".into(),
            command_type: "Get".into(),
            arguments: HashMap::new(),
            context: Vec::new(),
        },
        layout: None,
        on_enter: None,
        link: None,
    }
}

/// Opens the next/previous email of the current one's thread, replacing the
/// current buffer's content with that email's message view.
fn thread_navigate(delta: i64) {
    /// `Ok(Some(email_id))` opens that email, `Ok(None)` signals the thread
    /// boundary.
    type NavResult = anyhow::Result<Option<String>>;

    let buffer = api::get_current_buf();

    let config = match Config::read_from_file(None) {
        Ok(config) => config,
        Err(err) => {
            nvim_oxi::print!("failed to read config: {err}");
            return;
        }
    };

    let context = match resolve_cursor_email_context() {
        Ok(context) => context,
        Err(err) => {
            nvim_oxi::print!("{err:#}");
            return;
        }
    };

    let current_email_id = context.email_ids[0].clone();
    let account_id = context.account_id.clone();
    let folder_id = context.folder_id.clone();

    let account_id_for_schedule = context.account_id;
    let folder_id_for_schedule = context.folder_id;

    let config_for_schedule = config.clone();
    let shared_result = Arc::new(Mutex::<Option<NavResult>>::new(None));
    let shared_result_for_async = Arc::clone(&shared_result);

    let Some(async_handle) = new_async_handle(move || {
        let mut lock = shared_result.lock().unwrap();
        if let Some(result) = lock.take() {
            let buffer = buffer.clone();
            let config = config_for_schedule.clone();
            let account_id = account_id_for_schedule.clone();
            let folder_id = folder_id_for_schedule.clone();
            nvim_oxi::schedule(move |()| match result {
                Ok(Some(email_id)) => {
                    let mut component = email_message_component();
                    component.context.context = vec![
                        UiViewComponentContextContext::AccountId(account_id),
                        UiViewComponentContextContext::FolderId(folder_id),
                        UiViewComponentContextContext::EmailId(email_id),
                    ];
                    load_into(component, config, buffer, None);
                }
                Ok(None) => {
                    nvim_oxi::print!(
                        "Already at the {} email in the thread.",
                        if delta > 0 { "last" } else { "first" }
                    );
                }
                Err(err) => nvim_oxi::print!("{err:#}"),
            });
        }
    }) else {
        return;
    };

    ASYNC_RUNTIME.spawn(async move {
        let result: NavResult = async {
            let provider = config.to_provider()?;
            let threaded = provider
                .list_threads(&account_id, &current_email_id, Some(&folder_id))
                .await?;

            let current_index = threaded
                .iter()
                .position(|email| email.email().id() == current_email_id)
                .ok_or_else(|| anyhow::anyhow!("current email not found in its thread"))?;

            let target_index = i64::try_from(current_index)
                .unwrap_or(i64::MAX)
                .saturating_add(delta);

            if target_index.is_negative() {
                return Ok(None);
            }

            let Ok(target_index) = usize::try_from(target_index) else {
                return Ok(None);
            };

            let Some(target) = threaded.get(target_index) else {
                return Ok(None);
            };

            Ok(Some(target.email().id().to_string()))
        }
        .await;

        *shared_result_for_async.lock().unwrap() = Some(result);
        send_async(&async_handle);
    });
}

pub struct EmailThread;

impl UserCommand for EmailThread {
    const NAME: Name = Name::new("MailEmailThread");
    const DESCRIPTION: &'static str = "Show the details to the current email's thread";

    fn callback(_: CommandArgs) {
        open_thread(false);
    }
}

pub struct EmailThreadList;

impl UserCommand for EmailThreadList {
    const NAME: Name = Name::new("MailEmailThreadList");
    const DESCRIPTION: &'static str = "List the emails of the current email's thread";

    fn callback(_: CommandArgs) {
        open_thread(true);
    }
}

pub struct EmailThreadNext;

impl UserCommand for EmailThreadNext {
    const NAME: Name = Name::new("MailEmailThreadNext");
    const DESCRIPTION: &'static str = "Go to the next email in the thread";

    fn callback(_: CommandArgs) {
        thread_navigate(1);
    }
}

pub struct EmailThreadPrevious;

impl UserCommand for EmailThreadPrevious {
    const NAME: Name = Name::new("MailEmailThreadPrevious");
    const DESCRIPTION: &'static str = "Go to the previous email in the thread";

    fn callback(_: CommandArgs) {
        thread_navigate(-1);
    }
}
