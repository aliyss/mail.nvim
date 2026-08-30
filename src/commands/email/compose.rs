//! Composing and sending emails: `:MailEmailCreate`, `:MailEmailReply`,
//! `:MailEmailReplyAll`, `:MailEmailForward` and `:MailEmailSend`.
//!
//! Create opens an empty compose buffer; Reply/ReplyAll/Forward pre-fill it
//! with the recipients, subject and a quoted original message. The buffer
//! carries the same metadata block as every other mail buffer (so the
//! account survives), followed by an RFC 822 header block (`To:`, `Cc:`,
//! `Bcc:`, `Subject:`) and the body. `:MailEmailSend` parses that block,
//! builds a raw message (see [`crate::utils::message`]) and sends it through
//! the account's sending backend.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use serde_json::json;
use nvim_oxi::api::opts::OptionOpts;
use nvim_oxi::api::{self, Buffer};

use crate::api::config::Config;
use crate::api::file::TryFile;
use crate::api::config::ui::view::{
    UiViewComponent, UiViewComponentContext, UiViewComponentContextContext, UiViewComponentType,
};
use crate::api::email::EmailMessage;
use crate::api::account::commands::GetAccount;
use crate::api::email::commands::{GetEmail, SaveDraft, SendMessage};
use crate::commands::email::manage::{
    EmailActionContext, resolve_account_context, resolve_cursor_email_context,
};
use crate::commands::prelude::*;
use crate::providers::Provider;
use crate::utils::buffer::metadata::BufferMetadata;
use crate::utils::buffer::render::{FromBuffer, ToBuffer};
use crate::utils::message::{MessageParts, build_raw_message};
use crate::utils::render::{ASYNC_RUNTIME, new_async_handle, send_async};

/// What kind of compose buffer to open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ComposeKind {
    Reply,
    ReplyAll,
    Forward,
}

/// The resolved header fields of a compose buffer.
struct ComposeFields {
    to: String,
    cc: String,
    bcc: String,
    subject: String,
    body: String,
}

/// A single (account, folder, email) target for replying/forwarding.
#[derive(Debug, Clone)]
struct ComposeTarget {
    account_id: String,
    folder_id: String,
    email: String,
}

/// Resolves the email under the cursor as the target of a reply/forward.
fn resolve_compose_target() -> anyhow::Result<ComposeTarget> {
    let EmailActionContext {
        account_id,
        folder_id,
        email_ids,
    } = resolve_cursor_email_context()?;

    let Some(email) = email_ids.into_iter().next() else {
        anyhow::bail!("no email selected");
    };

    Ok(ComposeTarget {
        account_id,
        folder_id,
        email,
    })
}

/// Fetches the full message of `target`, so the compose buffer can be
/// pre-filled from it.
async fn fetch_target_message(
    config: &Config,
    target: &ComposeTarget,
) -> anyhow::Result<EmailMessage> {
    let provider = config.to_provider()?;
    let messages = provider
        .get_emails(
            &target.account_id,
            vec![target.email.as_str()],
            Some(target.folder_id.as_str()),
            None,
        )
        .await?;
    messages
        .into_iter()
        .next()
        .context("the email could not be fetched")
}

fn compose_subject(prefix: &str, subject: &str) -> String {
    let trimmed = subject.trim();
    if trimmed.len() >= prefix.len()
        && trimmed[..prefix.len()].eq_ignore_ascii_case(prefix)
    {
        trimmed.to_string()
    } else {
        format!("{prefix} {trimmed}")
    }
}

/// The date of the original message, formatted for the attribution line.
fn attribution_date(message: &EmailMessage) -> String {
    message.date.map_or_else(
        || "an unknown date".to_string(),
        |date| date.format("%a, %d %b %Y at %H:%M").to_string(),
    )
}

/// The sender line of the original message, for the attribution.
fn attribution_from(message: &EmailMessage) -> String {
    message
        .from
        .first()
        .map_or_else(|| "the sender".to_string(), ToString::to_string)
}

/// Quotes the body of the original message with `> ` prefixes.
fn quote_body(message: &EmailMessage) -> String {
    let mut quoted = format!("On {}, {} wrote:\n\n", attribution_date(message), attribution_from(message));
    for line in message.body_text.lines() {
        quoted.push_str("> ");
        quoted.push_str(line);
        quoted.push('\n');
    }
    quoted
}

/// The forwarded form of the original message.
fn forwarded_body(message: &EmailMessage) -> String {
    let mut body = String::new();
    writeln!(body, "---------- Forwarded message ----------").expect("write to string");
    writeln!(
        body,
        "From: {}",
        message
            .from
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join(", ")
    )
    .expect("write to string");
    writeln!(body, "Date: {}", attribution_date(message)).expect("write to string");
    writeln!(body, "Subject: {}\n", message.subject).expect("write to string");
    body.push_str(&message.body_text);
    body
}

/// The header fields and quoted body of the compose buffer for `kind`.
fn compose_parts(kind: ComposeKind, message: &EmailMessage) -> (ComposeFields, Option<String>) {
    match kind {
        ComposeKind::Reply => (
            ComposeFields {
                to: message
                    .from
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", "),
                cc: String::new(),
                bcc: String::new(),
                subject: compose_subject("Re:", &message.subject),
                body: String::new(),
            },
            Some(quote_body(message)),
        ),
        ComposeKind::ReplyAll => {
            // The original recipients join the reply: the original `To` and
            // `Cc` (minus the sender's own address, which is whoever the
            // account is configured as).
            let mut cc: Vec<String> = message
                .to
                .iter()
                .chain(message.cc.iter())
                .map(ToString::to_string)
                .collect();
            cc.dedup();
            (
                ComposeFields {
                    to: message
                        .from
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(", "),
                    cc: cc.join(", "),
                    bcc: String::new(),
                    subject: compose_subject("Re:", &message.subject),
                    body: String::new(),
                },
                Some(quote_body(message)),
            )
        }
        ComposeKind::Forward => (
            ComposeFields {
                to: String::new(),
                cc: String::new(),
                bcc: String::new(),
                subject: compose_subject("Fwd:", &message.subject),
                body: String::new(),
            },
            Some(forwarded_body(message)),
        ),
    }
}

/// The metadata component stashed in a brand-new compose buffer: the sending
/// account (and folder when known), so `:MailEmailSend` can rebuild the raw
/// message from the edited buffer alone.
fn create_component(account_id: &str, folder_id: Option<&str>) -> UiViewComponent {
    let mut context = vec![UiViewComponentContextContext::AccountId(account_id.to_string())];
    if let Some(folder_id) = folder_id {
        context.push(UiViewComponentContextContext::FolderId(folder_id.to_string()));
    }

    UiViewComponent {
        id: "email-compose".into(),
        name: "Compose".into(),
        component_type: UiViewComponentType::Other("compose".into()),
        context: UiViewComponentContext {
            command_group: "Email".into(),
            command_type: "Send".into(),
            arguments: HashMap::new(),
            context,
        },
        layout: None,
        on_enter: None,
        link: None,
    }
}

/// The metadata component stashed in the compose buffer: the account/folder/
/// email context plus the reply threading headers, so `:MailEmailSend` can
/// rebuild the raw message from the edited buffer alone.
fn compose_component(
    target: &ComposeTarget,
    message: &EmailMessage,
) -> UiViewComponent {
    let mut arguments = HashMap::new();
    arguments.insert(
        "in_reply_to".to_string(),
        json!(message.id.clone()),
    );
    arguments.insert(
        "references".to_string(),
        json!(message.id.clone()),
    );

    UiViewComponent {
        id: "email-compose".into(),
        name: "Compose".into(),
        component_type: UiViewComponentType::Other("compose".into()),
        context: UiViewComponentContext {
            command_group: "Email".into(),
            command_type: "Send".into(),
            arguments,
            context: vec![
                UiViewComponentContextContext::AccountId(target.account_id.clone()),
                UiViewComponentContextContext::FolderId(target.folder_id.clone()),
                UiViewComponentContextContext::EmailId(target.email.clone()),
            ],
        },
        layout: None,
        on_enter: None,
        link: None,
    }
}

/// Opens an editable compose buffer with the given metadata and content.
fn open_compose_buffer(
    component: UiViewComponent,
    fields: &ComposeFields,
    body: &str,
) -> anyhow::Result<()> {
    let opts = OptionOpts::builder().scope(OptionScope::Local).build();
    let mut buffer = api::create_buf(true, true)?;
    api::set_current_buf(&buffer)?;

    for (name, value) in [("filetype", "mail-compose"), ("buftype", "nofile")] {
        api::set_option_value(name, value, &opts)?;
    }
    api::set_option_value("swapfile", false, &opts)?;

    // Keep the buffer editable: it is the user's message, not rendered data.
    api::set_option_value("modifiable", true, &opts)?;

    let metadata = BufferMetadata::new(component).to_buffer(&mut buffer, 0)?;

    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("To: {}", fields.to));
    if !fields.cc.is_empty() {
        lines.push(format!("Cc: {}", fields.cc));
    }
    if !fields.bcc.is_empty() {
        lines.push(format!("Bcc: {}", fields.bcc));
    }
    lines.push(format!("Subject: {}", fields.subject));
    lines.push(String::new());
    lines.extend(body.lines().map(str::to_string));

    buffer.set_lines(metadata.line_count..metadata.line_count, true, lines)?;
    api::set_option_value("modifiable", true, &opts)?;

    crate::utils::syntax::apply(&buffer)?;

    // Park the cursor at the top of the body: the metadata block, then the
    // `To`/`Cc`/`Bcc`/`Subject` headers, the blank separator line, then the
    // first body line.
    let body_start = metadata.line_count
        + 1 // To
        + usize::from(!fields.cc.is_empty())
        + usize::from(!fields.bcc.is_empty())
        + 1 // Subject
        + 1 // blank separator
        + 1; // first body line (1-based)
    api::get_current_win().set_cursor(body_start, 0)?;

    Ok(())
}

/// The account a brand-new message is sent from: the account (and folder, if
/// any) of the current buffer — folder or email list, reading pane — or,
/// when there is none, the default account of the configuration.
fn resolve_create_account(config: &Config) -> anyhow::Result<(String, Option<String>)> {
    if let Ok(context) = resolve_account_context() {
        return Ok(context);
    }

    let provider = config.to_provider()?;
    Ok((provider.get_default_account()?.name().to_string(), None))
}

/// Opens an empty compose buffer for a brand-new message. Synchronous: there
/// is nothing to fetch.
pub(crate) fn compose_create(config: &Config) {
    let (account_id, folder_id) = match resolve_create_account(config) {
        Ok(context) => context,
        Err(err) => {
            nvim_oxi::print!("{err:#}");
            return;
        }
    };

    let component = create_component(&account_id, folder_id.as_deref());
    let fields = ComposeFields {
        to: String::new(),
        cc: String::new(),
        bcc: String::new(),
        subject: String::new(),
        body: String::new(),
    };

    if let Err(err) = open_compose_buffer(component, &fields, "") {
        nvim_oxi::print!("failed to open compose buffer: {err:#}");
    }
}

/// Shared plumbing of the reply/forward commands: resolve the target, fetch
/// the message on the async runtime and open the compose buffer on the main
/// thread.
fn compose(kind: ComposeKind) {
    let config = match Config::read_from_file(None) {
        Ok(config) => config,
        Err(err) => {
            nvim_oxi::print!("failed to read config: {err:#}");
            return;
        }
    };
    compose_with_config(kind, config);
}

/// Like [`compose`] but with an explicit configuration (used by the tests,
/// which drive the flow with the fake provider).
pub(crate) fn compose_with_config(kind: ComposeKind, config: Config) {
    let target = match resolve_compose_target() {
        Ok(target) => target,
        Err(err) => {
            nvim_oxi::print!("{err:#}");
            return;
        }
    };

    let shared_result = Arc::new(Mutex::<Option<anyhow::Result<EmailMessage>>>::new(None));
    let shared_result_for_async = Arc::clone(&shared_result);

    let target_for_closure = target.clone();
    let Some(async_handle) = new_async_handle(move || {
        if let Some(result) = shared_result.lock().unwrap().take() {
            let target = target_for_closure.clone();
            nvim_oxi::schedule(move |()| match result {
                Ok(message) => {
                    let (fields, body) = compose_parts(kind, &message);
                    let component = compose_component(&target, &message);
                    if let Err(err) =
                        open_compose_buffer(component, &fields, body.as_deref().unwrap_or(""))
                    {
                        nvim_oxi::print!("failed to open compose buffer: {err:#}");
                    }
                }
                Err(err) => nvim_oxi::print!("failed to fetch email: {err:#}"),
            });
        }
    }) else {
        return;
    };

    ASYNC_RUNTIME.spawn(async move {
        let result = fetch_target_message(&config, &target).await;
        *shared_result_for_async.lock().unwrap() = Some(result);
        send_async(&async_handle);
    });
}

pub struct EmailCreate;

impl UserCommand for EmailCreate {
    const NAME: Name = Name::new("MailEmailCreate");
    const DESCRIPTION: &'static str = "Create a new email";

    fn callback(_: CommandArgs) {
        let config = match Config::read_from_file(None) {
            Ok(config) => config,
            Err(err) => {
                nvim_oxi::print!("failed to read config: {err:#}");
                return;
            }
        };
        compose_create(&config);
    }
}

pub struct EmailReply;

impl UserCommand for EmailReply {
    const NAME: Name = Name::new("MailEmailReply");
    const DESCRIPTION: &'static str = "Reply to an email";

    fn callback(_: CommandArgs) {
        compose(ComposeKind::Reply);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(subject: &str) -> EmailMessage {
        EmailMessage {
            id: "<abc@example.com>".into(),
            thread_id: None,
            subject: subject.into(),
            from: vec![crate::api::contact::Address::Individual(
                crate::api::contact::Contact {
                    name: Some("Sender".into()),
                    email: "sender@example.com".into(),
                },
            )],
            to: vec![],
            cc: vec![],
            bcc: vec![],
            date: None,
            body_text: "first line\nsecond line".into(),
            body_html: None,
            attachment_ids: vec![],
        }
    }

    #[test]
    fn reply_subject_is_prefixed_once() {
        assert_eq!(compose_subject("Re:", "Hello"), "Re: Hello");
        assert_eq!(compose_subject("Re:", "Re: Hello"), "Re: Hello");
        assert_eq!(compose_subject("Re:", "re: hello"), "re: hello");
        assert_eq!(compose_subject("Fwd:", "Re: Hello"), "Fwd: Re: Hello");
    }

    #[test]
    fn reply_targets_the_sender() {
        let (fields, body) = compose_parts(ComposeKind::Reply, &message("Hello"));
        assert_eq!(fields.to, "Sender <sender@example.com>");
        assert_eq!(fields.subject, "Re: Hello");
        let body = body.expect("reply quotes the original");
        assert!(body.contains("> first line"));
        assert!(body.contains("> second line"));
    }

    #[test]
    fn forward_leaves_to_empty_and_prepends_subject() {
        let (fields, body) = compose_parts(ComposeKind::Forward, &message("Hello"));
        assert!(fields.to.is_empty());
        assert_eq!(fields.subject, "Fwd: Hello");
        let body = body.expect("forward includes the original");
        assert!(body.contains("---------- Forwarded message ----------"));
        assert!(body.contains("first line"));
    }
}

pub struct EmailReplyAll;

impl UserCommand for EmailReplyAll {
    const NAME: Name = Name::new("MailEmailReplyAll");
    const DESCRIPTION: &'static str = "Reply to all recipients of an email";

    fn callback(_: CommandArgs) {
        compose(ComposeKind::ReplyAll);
    }
}

pub struct EmailForward;

impl UserCommand for EmailForward {
    const NAME: Name = Name::new("MailEmailForward");
    const DESCRIPTION: &'static str = "Forward an email";

    fn callback(_: CommandArgs) {
        compose(ComposeKind::Forward);
    }
}

/// Parses the header block of a compose buffer (the lines between the
/// metadata and the first blank line) and returns the fields plus the body.
///
/// Unlike sending, drafts may be saved with empty recipients or subjects, so
/// no field is required here.
fn parse_compose_fields(buffer: &Buffer, metadata: &BufferMetadata) -> anyhow::Result<ComposeFields> {
    let lines: Vec<String> = buffer
        .get_lines(metadata.line_count.., true)
        .map_err(|_| anyhow::anyhow!("failed to read lines from buffer"))?
        .map(|line| line.to_string())
        .collect();

    let mut headers: HashMap<String, String> = HashMap::new();
    let mut body_start = lines.len();

    for (index, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            body_start = index + 1;
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(
                key.trim().to_ascii_lowercase(),
                value.trim().to_string(),
            );
        }
    }

    Ok(ComposeFields {
        to: headers.remove("to").unwrap_or_default(),
        cc: headers.remove("cc").unwrap_or_default(),
        bcc: headers.remove("bcc").unwrap_or_default(),
        subject: headers.remove("subject").unwrap_or_default(),
        body: lines[body_start..].join("\n"),
    })
}

/// Everything a compose buffer contributes to a raw message: the editable
/// fields, the sending account and the reply threading headers.
struct ComposeMessage {
    fields: ComposeFields,
    account_id: String,
    in_reply_to: Option<String>,
    references: Option<String>,
}

/// Parses the current buffer as a compose buffer (metadata + fields).
fn read_compose_message(buffer: &Buffer) -> anyhow::Result<ComposeMessage> {
    let metadata = BufferMetadata::from_buffer(buffer, None)?;

    if metadata.component.context.command_group != "Email"
        || metadata.component.context.command_type != "Send"
    {
        anyhow::bail!("not in a compose buffer: start one with :MailEmailCreate or :MailEmailReply");
    }

    let fields = parse_compose_fields(buffer, &metadata)?;
    let account_id = metadata
        .component
        .context
        .get_required_context("account_id", Some("no account found in compose buffer"))?
        .as_str()
        .to_string();

    let arguments = &metadata.component.context.arguments;
    let in_reply_to = arguments
        .get("in_reply_to")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let references = arguments
        .get("references")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);

    Ok(ComposeMessage {
        fields,
        account_id,
        in_reply_to,
        references,
    })
}

/// Builds the raw message of `message` from the account's sender address.
fn build_message(
    provider: &impl Provider,
    message: &ComposeMessage,
) -> anyhow::Result<Vec<u8>> {
    let from = provider.get_sender_address(&message.account_id)?;
    let parts = MessageParts {
        from,
        to: message.fields.to.clone(),
        cc: message.fields.cc.clone(),
        bcc: message.fields.bcc.clone(),
        subject: message.fields.subject.clone(),
        body: message.fields.body.clone(),
        in_reply_to: message.in_reply_to.clone(),
        references: message.references.clone(),
    };
    Ok(build_raw_message(&parts))
}

/// Runs `action` (which builds its own provider from the configuration it
/// captures) on the async runtime and schedules the result back to the main
/// thread: `success` is printed and the compose buffer is closed on success,
/// `failure` on error.
fn spawn_compose_action<F, Fut>(
    buffer: Buffer,
    success: &'static str,
    failure: &'static str,
    action: F,
) where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
{
    let buffer_for_async = buffer;
    let shared_result = Arc::new(Mutex::<Option<anyhow::Result<()>>>::new(None));
    let shared_result_for_async = Arc::clone(&shared_result);

    let Some(async_handle) = new_async_handle(move || {
        if let Some(result) = shared_result.lock().unwrap().take() {
            let buffer_for_schedule = buffer_for_async.clone();
            nvim_oxi::schedule(move |()| match result {
                Ok(()) => {
                    nvim_oxi::print!("{success}");
                    // The compose buffer did its job; close it. The
                    // `:bdelete` command is a no-op in some headless
                    // contexts, so delete through the API.
                    let opts = nvim_oxi::api::opts::BufDeleteOpts::builder()
                        .force(true)
                        .build();
                    if let Err(err) = buffer_for_schedule.delete(&opts) {
                        nvim_oxi::print!("failed to close compose buffer: {err}");
                    }
                }
                Err(err) => nvim_oxi::print!("{failure}: {err:#}"),
            });
        }
    }) else {
        return;
    };

    ASYNC_RUNTIME.spawn(async move {
        let result = action().await;
        *shared_result_for_async.lock().unwrap() = Some(result);
        send_async(&async_handle);
    });
}

/// Sends the message of the current compose buffer, requiring a recipient
/// and a subject.
pub(crate) fn send_with_config(config: Config) {
    let buffer = api::get_current_buf();

    let message = match read_compose_message(&buffer) {
        Ok(message) => message,
        Err(err) => {
            nvim_oxi::print!("{err:#}");
            return;
        }
    };

    if message.fields.to.trim().is_empty() {
        nvim_oxi::print!("no recipient: fill in the `To:` line of the compose buffer");
        return;
    }
    if message.fields.subject.trim().is_empty() {
        nvim_oxi::print!("no subject: fill in the `Subject:` line of the compose buffer");
        return;
    }

    spawn_compose_action(buffer, "Message sent!", "failed to send message", move || async move {
        let provider = config.to_provider()?;
        let raw = build_message(&provider, &message)?;
        provider.send_message(&message.account_id, raw).await
    });
}

/// Saves the message of the current compose buffer as a draft. Unlike
/// sending, drafts may be saved before the recipients and subject are filled
/// in.
pub(crate) fn save_draft_with_config(config: Config) {
    let buffer = api::get_current_buf();

    let message = match read_compose_message(&buffer) {
        Ok(message) => message,
        Err(err) => {
            nvim_oxi::print!("{err:#}");
            return;
        }
    };

    spawn_compose_action(buffer, "Draft saved", "failed to save draft", move || async move {
        let provider = config.to_provider()?;
        let raw = build_message(&provider, &message)?;
        provider.save_draft(&message.account_id, raw).await
    });
}

pub struct EmailSend;

impl UserCommand for EmailSend {
    const NAME: Name = Name::new("MailEmailSend");
    const DESCRIPTION: &'static str = "Send the email in the current compose buffer";

    fn callback(_: CommandArgs) {
        let config = match Config::read_from_file(None) {
            Ok(config) => config,
            Err(err) => {
                nvim_oxi::print!("failed to read config: {err:#}");
                return;
            }
        };
        send_with_config(config);
    }
}

pub struct EmailSaveAsDraft;

impl UserCommand for EmailSaveAsDraft {
    const NAME: Name = Name::new("MailEmailSaveAsDraft");
    const DESCRIPTION: &'static str = "Save the email in the current compose buffer as a draft";

    fn callback(_: CommandArgs) {
        let config = match Config::read_from_file(None) {
            Ok(config) => config,
            Err(err) => {
                nvim_oxi::print!("failed to read config: {err:#}");
                return;
            }
        };
        save_draft_with_config(config);
    }
}
