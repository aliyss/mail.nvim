//! Shared helpers for live command-line completion of command arguments.
//!
//! Each [`UserCommand`](crate::commands::UserCommand) can implement
//! [`complete`](crate::commands::UserCommand::complete) to offer dynamic
//! candidates; this module provides the underlying data: account names from
//! the Himalaya config, cached folder/email ids, static flag names and the
//! current buffer's account/folder context.

use nvim_oxi::api;

use crate::api::config::Config;
use crate::api::config::ui::view::UiViewComponentContextContext;
use crate::api::file::TryFile;
use crate::providers::himalaya::HimalayaProvider;
use crate::utils::buffer::metadata::BufferMetadata;
use crate::utils::buffer::render::FromBuffer;
use crate::utils::completion;

/// The `fargs` of a command line: the whitespace-separated tokens after the
/// command name.
pub(crate) fn cmdline_args(cmd_line: &str) -> Vec<String> {
    cmd_line
        .split_whitespace()
        .skip(1)
        .map(String::from)
        .collect()
}

/// Keeps only the candidates starting with `arg_lead` (case-insensitive),
/// sorted. Neovim's `customlist` completion shows exactly what is returned.
#[must_use]
pub fn filter(arg_lead: &str, candidates: Vec<String>) -> Vec<String> {
    let lead = arg_lead.to_ascii_lowercase();
    let mut matches: Vec<String> = candidates
        .into_iter()
        .filter(|candidate| candidate.to_ascii_lowercase().starts_with(&lead))
        .collect();
    matches.sort();
    matches.dedup();
    matches
}

/// The configured account names, in configuration order.
#[must_use]
pub fn accounts() -> Vec<String> {
    let config = match Config::read_from_file(None) {
        Ok(config) => config,
        Err(_) => return Vec::new(),
    };
    let provider = match HimalayaProvider::from_config(&config) {
        Ok(provider) => provider,
        Err(_) => return Vec::new(),
    };
    provider.config().accounts.keys().cloned().collect()
}

/// Whether `name` is one of the configured accounts.
#[must_use]
pub fn is_account(name: &str) -> bool {
    accounts().iter().any(|account| account == name)
}

/// The standard email flag names.
#[must_use]
pub fn flags() -> Vec<String> {
    ["seen", "answered", "flagged", "deleted", "draft"]
        .into_iter()
        .map(String::from)
        .collect()
}

/// The `t`/`f` variants accepted by boolean command arguments.
#[must_use]
pub fn booleans() -> Vec<String> {
    vec!["t".into(), "f".into(), "true".into(), "false".into()]
}

/// The account and folder of the current buffer, when its metadata carries
/// them.
#[must_use]
pub fn current_context() -> (Option<String>, Option<String>) {
    let buffer = api::get_current_buf();
    let Ok(metadata) = BufferMetadata::from_buffer(&buffer, None) else {
        return (None, None);
    };

    let mut account = None;
    let mut folder = None;
    for entry in &metadata.component.context.context {
        match entry {
            UiViewComponentContextContext::AccountId(id) => account = Some(id.clone()),
            UiViewComponentContextContext::FolderId(id) => folder = Some(id.clone()),
            _ => {}
        }
    }
    (account, folder)
}

/// The account driving a command's completion: the account named in the
/// command line (a previous argument) if any, otherwise the current buffer's
/// account.
#[must_use]
pub fn account_from(cmd_line: &str) -> Option<String> {
    cmdline_args(cmd_line)
        .first()
        .filter(|arg| is_account(arg))
        .cloned()
        .or_else(|| current_context().0)
}

/// Cached folder names for `account`, or for every account when `None`.
#[must_use]
pub fn folder_names(account: Option<&str>) -> Vec<String> {
    completion::folder_names(account)
}

/// Cached email ids of `account`, restricted to `folder` when given.
#[must_use]
pub fn email_ids(account: &str, folder: Option<&str>) -> Vec<String> {
    completion::email_ids(account, folder)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_is_case_insensitive_and_sorted() {
        let candidates = vec![
            "Trash".to_string(),
            "INBOX".to_string(),
            "trash".to_string(),
        ];
        assert_eq!(
            filter("tr", candidates),
            vec!["Trash".to_string(), "trash".to_string()]
        );
        assert_eq!(
            filter("", vec!["b".to_string(), "a".to_string()]),
            vec!["a".to_string(), "b".to_string()]
        );
    }

    #[test]
    fn filter_dedups() {
        assert_eq!(
            filter("", vec!["a".into(), "a".into()]),
            vec!["a".to_string()]
        );
    }

    #[test]
    fn cmdline_args_skips_the_command_name() {
        assert_eq!(cmdline_args("MailEmailMove Tr"), vec!["Tr".to_string()]);
        assert_eq!(cmdline_args("MailEmailMove"), Vec::<String>::new());
    }
}
