//! Mutating email commands (flags, read status, delete, move, copy).
//!
//! These commands operate on the explicitly selected emails of the current
//! list (see [`crate::commands::email::selection`]), falling back to the
//! email under the cursor when nothing is selected. The account, folder and
//! email ids are resolved from the current buffer's metadata and its rows,
//! falling back to the component context of the file view.

use std::future::Future;
use std::sync::{Arc, Mutex};

use anyhow::Context;

use crate::api::config::Config;
use crate::api::config::ui::view::{
    UiViewComponent, UiViewComponentContext, UiViewComponentContextContext, UiViewComponentType,
};
use crate::api::email::Email;
use crate::api::email::EmailFlag;
use crate::api::email::commands::{
    AddEmailFlags, CopyEmails, DeleteEmails, MoveEmails, RemoveEmailFlags, SetEmailFlags,
};
use crate::api::file::TryFile;
use crate::commands::completion;
use crate::commands::prelude::*;
use crate::utils::buffer::metadata::BufferMetadata;
use crate::utils::buffer::render::FromBuffer;
use crate::utils::confirm::{self, RiskLevel};
use crate::utils::render::table::context::fetch_row_email;
use crate::utils::render::table::render::Table;
use crate::utils::render::{ASYNC_RUNTIME, get_context, load_into, new_async_handle, send_async};
use crate::utils::selection;

/// The resolved target of an email action: which account, folder and emails.
pub(crate) struct EmailActionContext {
    pub account_id: String,
    pub folder_id: String,
    pub email_ids: Vec<String>,
}

fn email_get_component() -> UiViewComponent {
    UiViewComponent {
        id: "email-action-context".into(),
        name: "EmailAction".into(),
        component_type: UiViewComponentType::File,
        context: UiViewComponentContext {
            command_group: "Email".into(),
            command_type: "Get".into(),
            arguments: std::collections::HashMap::new(),
            context: Vec::new(),
        },
        layout: None,
        on_enter: None,
        link: None,
    }
}

/// Resolves the account, folder and email ids the action should target.
///
/// When emails are explicitly selected in the current list, the action
/// applies to all of them; otherwise it falls back to the email under the
/// cursor.
///
/// # Errors
///
/// Returns an error if the account, folder or email cannot be determined from
/// the current buffer.
pub(crate) fn resolve_email_context() -> anyhow::Result<EmailActionContext> {
    resolve_email_context_inner(true)
}

/// Like [`resolve_email_context`] but always targets the email under the
/// cursor, ignoring the selection (used by thread navigation).
///
/// # Errors
///
/// Returns an error if the account, folder or email cannot be determined from
/// the current buffer.
pub(crate) fn resolve_cursor_email_context() -> anyhow::Result<EmailActionContext> {
    resolve_email_context_inner(false)
}

fn resolve_email_context_inner(use_selection: bool) -> anyhow::Result<EmailActionContext> {
    let buffer = api::get_current_buf();
    let component = email_get_component();
    let context = get_context(Some(buffer.clone()), &component)?;

    let mut account_id = None;
    let mut folder_id = None;
    let mut email_ids = Vec::new();

    for entry in &context {
        match entry {
            UiViewComponentContextContext::AccountId(id) => account_id = Some(id.clone()),
            UiViewComponentContextContext::FolderId(id) => folder_id = Some(id.clone()),
            UiViewComponentContextContext::EmailId(id) => email_ids.push(id.clone()),
        }
    }

    let account_id = account_id.context("no account selected")?;
    let folder_id = folder_id.context("no folder selected")?;

    // An explicit multi-select wins over the email under the cursor.
    if use_selection {
        let selected = selection::selected_ids(&buffer);
        if !selected.is_empty() {
            email_ids = selected.into_iter().collect();
        }
    }

    if email_ids.is_empty() {
        anyhow::bail!("no email selected");
    }

    Ok(EmailActionContext {
        account_id,
        folder_id,
        email_ids,
    })
}

/// Resolves the account (and, when available, folder) of the current buffer
/// without requiring an email under the cursor — used by new-message
/// composing, which only needs to know who is sending.
///
/// # Errors
///
/// Returns an error when the account cannot be determined from the current
/// buffer.
pub(crate) fn resolve_account_context() -> anyhow::Result<(String, Option<String>)> {
    let buffer = api::get_current_buf();
    let component = email_get_component();
    let context = get_context(Some(buffer), &component)?;

    let mut account_id = None;
    let mut folder_id = None;

    for entry in &context {
        match entry {
            UiViewComponentContextContext::AccountId(id) => account_id = Some(id.clone()),
            UiViewComponentContextContext::FolderId(id) => folder_id = Some(id.clone()),
            UiViewComponentContextContext::EmailId(_) => {}
        }
    }

    let account_id = account_id.context("no account selected")?;
    Ok((account_id, folder_id))
}

/// Fetches the emails the current action targets, so callers can inspect
/// their flags: the explicit selection, or the email under the cursor.
fn resolve_target_emails() -> Vec<Email> {
    let buffer = api::get_current_buf();
    let Ok(metadata) = BufferMetadata::from_buffer(&buffer, None) else {
        return Vec::new();
    };

    if metadata.component.context.command_group == "Email"
        && matches!(
            metadata.component.context.command_type.as_str(),
            "List" | "Thread"
        )
    {
        let selected = selection::selected_ids(&buffer);
        if selected.is_empty() {
            // Fall back to the row under the cursor.
            return fetch_row_email(&buffer, metadata.line_count)
                .map(|email| vec![email])
                .unwrap_or_default();
        }

        // Prefer the pane's cached data (the same emails that rendered the
        // table, so a truncated header cannot hide them), then fall back to
        // re-parsing the rendered table.
        let emails = crate::utils::render::cached_pane_data(&buffer)
            .and_then(|data| match data {
                crate::utils::render::ComponentData::Emails(emails) => Some(emails),
                crate::utils::render::ComponentData::Threads(emails) => Some(
                    emails
                        .into_iter()
                        .map(crate::api::email::ThreadedEmail::into_email)
                        .collect(),
                ),
                _ => None,
            })
            .or_else(|| {
                Table::<Vec<Email>>::from_buffer(&buffer, Some(metadata.line_count))
                    .ok()
                    .map(|table| table.data)
            });

        emails
            .into_iter()
            .flatten()
            .filter(|email| selected.contains(email.id()))
            .collect()
    } else {
        Vec::new()
    }
}

/// Prompts the user for a value, returning `None` when cancelled or empty.
fn prompt(message: &str) -> Option<String> {
    let value: String = api::call_function("input", (message, "")).unwrap_or_default();
    let value = value.trim().to_string();
    (!value.is_empty()).then_some(value)
}

/// Runs `f` on the async runtime and schedules the result back to the main
/// thread, where it is printed and the current email list is refreshed.
///
/// The account, folder and email ids are resolved from the current buffer.
pub(crate) fn spawn_email_action<F, Fut>(f: F)
where
    F: FnOnce(EmailActionContext, Config) -> Fut + Send + 'static,
    Fut: Future<Output = anyhow::Result<String>> + Send + 'static,
{
    let context = match resolve_email_context() {
        Ok(context) => context,
        Err(err) => {
            nvim_oxi::print!("{err:#}");
            return;
        }
    };

    spawn_email_action_with_context(context, f);
}

/// Like [`spawn_email_action`] but with a pre-resolved context, used after a
/// confirmation popup accepted the action (the popup stole focus, so the
/// context must be resolved before it opens).
pub(crate) fn spawn_email_action_with_context<F, Fut>(context: EmailActionContext, f: F)
where
    F: FnOnce(EmailActionContext, Config) -> Fut + Send + 'static,
    Fut: Future<Output = anyhow::Result<String>> + Send + 'static,
{
    let config = match Config::read_from_file(None) {
        Ok(config) => config,
        Err(err) => {
            nvim_oxi::print!("failed to read config: {err}");
            return;
        }
    };

    let buffer = api::get_current_buf();

    let config_for_schedule = config.clone();
    let shared_result = Arc::new(Mutex::<Option<anyhow::Result<String>>>::new(None));
    let shared_result_for_async = Arc::clone(&shared_result);

    let Some(async_handle) = new_async_handle(move || {
        let mut lock = shared_result.lock().unwrap();
        if let Some(result) = lock.take() {
            let config = config_for_schedule.clone();
            let buffer = buffer.clone();
            nvim_oxi::schedule(move |()| match result {
                Ok(message) => {
                    nvim_oxi::print!("{message}");
                    // The emails are gone/updated: drop the stale selection.
                    selection::clear(&buffer);
                    refresh_current_email_list(config);
                }
                Err(err) => nvim_oxi::print!("{err:#}"),
            });
        }
    }) else {
        return;
    };

    ASYNC_RUNTIME.spawn(async move {
        let result = f(context, config).await;
        *shared_result_for_async.lock().unwrap() = Some(result);
        send_async(&async_handle);
    });
}

/// Resolves the action context and runs `action` with it, showing a
/// confirmation popup first when `config` requires one for `level`.
///
/// The context is resolved before the popup opens (the popup steals focus,
/// so it cannot be resolved from the current buffer afterwards). `lines`
/// builds the popup content from the resolved context.
fn with_resolved_context(
    level: RiskLevel,
    title: &str,
    lines: impl FnOnce(&EmailActionContext) -> Vec<String>,
    action: impl FnOnce(EmailActionContext) + Send + 'static,
) {
    let config = match Config::read_from_file(None) {
        Ok(config) => config,
        Err(err) => {
            nvim_oxi::print!("failed to read config: {err:#}");
            return;
        }
    };

    let context = match resolve_email_context() {
        Ok(context) => context,
        Err(err) => {
            nvim_oxi::print!("{err:#}");
            return;
        }
    };

    if confirm::requires_confirmation(&config, level) {
        confirm::confirm(title, lines(&context), Box::new(move || action(context)));
    } else {
        action(context);
    }
}

/// Re-renders the current email list in place after a successful mutation.
fn refresh_current_email_list(config: Config) {
    let buffer = api::get_current_buf();
    let Ok(metadata) = BufferMetadata::from_buffer(&buffer, None) else {
        return;
    };

    if metadata.component.context.command_group == "Email"
        && metadata.component.context.command_type == "List"
    {
        load_into(metadata.component, config, buffer, None);
    }
}

/// Parses a flag name into an [`EmailFlag`], defaulting to a custom flag.
fn parse_flag(value: &str) -> EmailFlag {
    match value.trim().to_ascii_lowercase().as_str() {
        "seen" => EmailFlag::Seen,
        "answered" => EmailFlag::Answered,
        "flagged" => EmailFlag::Flagged,
        "deleted" => EmailFlag::Deleted,
        "draft" => EmailFlag::Draft,
        custom => EmailFlag::Custom(custom.to_string()),
    }
}

fn ids_ref(context: &EmailActionContext) -> Vec<&str> {
    context.email_ids.iter().map(String::as_str).collect()
}

pub struct EmailToggleRead;

impl UserCommand for EmailToggleRead {
    const NAME: Name = Name::new("MailEmailToggleRead");
    const DESCRIPTION: &'static str = "Mark emails as read or unread";

    fn complete(arg_lead: &str, _cmd_line: &str, _cursor_pos: usize) -> Vec<String> {
        completion::filter(arg_lead, completion::booleans())
    }

    fn callback(args: CommandArgs) {
        let mark_read: Option<bool> = args.fargs.first().and_then(|arg| match arg.as_str() {
            "t" | "true" | "read" => Some(true),
            "f" | "false" | "unread" => Some(false),
            _ => None,
        });

        let targets = resolve_target_emails();
        let currently_seen = (!targets.is_empty()).then(|| {
            targets
                .iter()
                .all(|email| email.flags().contains(&EmailFlag::Seen))
        });

        spawn_email_action(move |context, config| async move {
            let provider = config.to_provider()?;
            let ids = ids_ref(&context);
            let mark_read = mark_read.unwrap_or_else(|| !currently_seen.unwrap_or(false));

            if mark_read {
                provider
                    .add_email_flags(
                        &context.account_id,
                        &context.folder_id,
                        ids,
                        vec![EmailFlag::Seen],
                    )
                    .await?;
                Ok("Marked email as read".to_string())
            } else {
                provider
                    .remove_email_flags(
                        &context.account_id,
                        &context.folder_id,
                        ids,
                        vec![EmailFlag::Seen],
                    )
                    .await?;
                Ok("Marked email as unread".to_string())
            }
        });
    }
}

pub struct EmailFlagAdd;

impl UserCommand for EmailFlagAdd {
    const NAME: Name = Name::new("MailEmailFlagAdd");
    const DESCRIPTION: &'static str = "Add a flag to an email";

    fn complete(arg_lead: &str, _cmd_line: &str, _cursor_pos: usize) -> Vec<String> {
        completion::filter(arg_lead, completion::flags())
    }

    fn callback(args: CommandArgs) {
        let Some(flag_name) = args.fargs.first().cloned().or_else(|| prompt("Flag: ")) else {
            nvim_oxi::print!("No flag provided.");
            return;
        };
        let flag = parse_flag(&flag_name);

        spawn_email_action(move |context, config| async move {
            let provider = config.to_provider()?;
            provider
                .add_email_flags(
                    &context.account_id,
                    &context.folder_id,
                    ids_ref(&context),
                    vec![flag],
                )
                .await?;
            Ok("Added flag to email".to_string())
        });
    }
}

pub struct EmailFlagRemove;

impl UserCommand for EmailFlagRemove {
    const NAME: Name = Name::new("MailEmailFlagRemove");
    const DESCRIPTION: &'static str = "Remove a flag from an email";

    fn complete(arg_lead: &str, _cmd_line: &str, _cursor_pos: usize) -> Vec<String> {
        completion::filter(arg_lead, completion::flags())
    }

    fn callback(args: CommandArgs) {
        let Some(flag_name) = args.fargs.first().cloned().or_else(|| prompt("Flag: ")) else {
            nvim_oxi::print!("No flag provided.");
            return;
        };
        let flag = parse_flag(&flag_name);

        spawn_email_action(move |context, config| async move {
            let provider = config.to_provider()?;
            provider
                .remove_email_flags(
                    &context.account_id,
                    &context.folder_id,
                    ids_ref(&context),
                    vec![flag],
                )
                .await?;
            Ok("Removed flag from email".to_string())
        });
    }
}

pub struct EmailFlagClear;

impl UserCommand for EmailFlagClear {
    const NAME: Name = Name::new("MailEmailFlagClear");
    const DESCRIPTION: &'static str = "Clear all flags from an email";

    fn callback(_: CommandArgs) {
        with_resolved_context(
            RiskLevel::Risky,
            "Clear flags",
            |context| {
                vec![
                    format!("Clear all flags of {} email(s)?", context.email_ids.len()),
                    String::new(),
                    "[y]es  [n]o".into(),
                ]
            },
            |context| {
                spawn_email_action_with_context(context, |context, config| async move {
                    let provider = config.to_provider()?;
                    provider
                        .set_email_flags(
                            &context.account_id,
                            &context.folder_id,
                            ids_ref(&context),
                            Vec::new(),
                        )
                        .await?;
                    Ok("Cleared flags from email".to_string())
                });
            },
        );
    }
}

pub struct EmailDelete;

impl UserCommand for EmailDelete {
    const NAME: Name = Name::new("MailEmailDelete");
    const DESCRIPTION: &'static str = "Delete emails";

    fn callback(_: CommandArgs) {
        with_resolved_context(
            RiskLevel::HighRisk,
            "Delete emails",
            |context| {
                vec![
                    format!("Delete {} email(s)?", context.email_ids.len()),
                    "This cannot be undone.".into(),
                    String::new(),
                    "[y]es  [n]o".into(),
                ]
            },
            |context| {
                spawn_email_action_with_context(context, |context, config| async move {
                    let provider = config.to_provider()?;
                    provider
                        .delete_emails(&context.account_id, &context.folder_id, ids_ref(&context))
                        .await?;
                    Ok(format!("Deleted {} email(s)", context.email_ids.len()))
                });
            },
        );
    }
}

pub struct EmailMove;

impl UserCommand for EmailMove {
    const NAME: Name = Name::new("MailEmailMove");
    const DESCRIPTION: &'static str = "Move emails to another folder";

    fn complete(arg_lead: &str, cmd_line: &str, _cursor_pos: usize) -> Vec<String> {
        // Folders of the account named on the command line (previous
        // argument), or of the current buffer's account.
        let account = completion::account_from(cmd_line);
        completion::filter(arg_lead, completion::folder_names(account.as_deref()))
    }

    fn callback(args: CommandArgs) {
        let Some(to_folder) = args
            .fargs
            .first()
            .cloned()
            .or_else(|| prompt("Move to folder: "))
        else {
            nvim_oxi::print!("No target folder provided.");
            return;
        };

        let to_folder_for_action = to_folder.clone();
        with_resolved_context(
            RiskLevel::Risky,
            "Move emails",
            move |context| {
                vec![
                    format!("Move {} email(s) to {to_folder}?", context.email_ids.len()),
                    String::new(),
                    "[y]es  [n]o".into(),
                ]
            },
            move |context| {
                spawn_email_action_with_context(context, move |context, config| async move {
                    let provider = config.to_provider()?;
                    provider
                        .move_emails(
                            &context.account_id,
                            &context.folder_id,
                            &to_folder_for_action,
                            ids_ref(&context),
                        )
                        .await?;
                    Ok(format!("Moved {} email(s)", context.email_ids.len()))
                });
            },
        );
    }
}

pub struct EmailCopy;

impl UserCommand for EmailCopy {
    const NAME: Name = Name::new("MailEmailCopy");
    const DESCRIPTION: &'static str = "Copy emails to another folder";

    fn complete(arg_lead: &str, cmd_line: &str, _cursor_pos: usize) -> Vec<String> {
        // Folders of the account named on the command line (previous
        // argument), or of the current buffer's account.
        let account = completion::account_from(cmd_line);
        completion::filter(arg_lead, completion::folder_names(account.as_deref()))
    }

    fn callback(args: CommandArgs) {
        let Some(to_folder) = args
            .fargs
            .first()
            .cloned()
            .or_else(|| prompt("Copy to folder: "))
        else {
            nvim_oxi::print!("No target folder provided.");
            return;
        };

        let to_folder_for_action = to_folder.clone();
        with_resolved_context(
            RiskLevel::Risky,
            "Copy emails",
            move |context| {
                vec![
                    format!("Copy {} email(s) to {to_folder}?", context.email_ids.len()),
                    String::new(),
                    "[y]es  [n]o".into(),
                ]
            },
            move |context| {
                spawn_email_action_with_context(context, move |context, config| async move {
                    let provider = config.to_provider()?;
                    provider
                        .copy_emails(
                            &context.account_id,
                            &context.folder_id,
                            &to_folder_for_action,
                            ids_ref(&context),
                        )
                        .await?;
                    Ok(format!("Copied {} email(s)", context.email_ids.len()))
                });
            },
        );
    }
}
