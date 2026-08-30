//! Confirmation popups for risky actions ("user handholding").
//!
//! When a mutating action requires confirmation, a floating window shows a
//! summary of the action with `[y]es` / `[n]o` bindings. `y` (or `<CR>`)
//! runs the pending action, `n`, `q` or `<Esc>` closes the popup and
//! discards it.

use std::sync::{Mutex, OnceLock};

use nvim_oxi::Object;
use nvim_oxi::api::opts::{OptionOpts, OptionScope, SetKeymapOpts};
use nvim_oxi::api::types::{
    Mode, WindowAnchor, WindowBorder, WindowConfig, WindowRelativeTo, WindowStyle, WindowTitle,
};
use nvim_oxi::api::{self, Buffer};

use crate::api::config::Config;

/// The filetype of confirmation popup buffers, used to identify and close them.
const CONFIRM_FILE_TYPE: &str = "mail-confirm";

/// A pending confirmation: the action to run when the user confirms.
type Pending = Box<dyn FnOnce() + Send>;

static PENDING: OnceLock<Mutex<Option<Pending>>> = OnceLock::new();

fn pending() -> &'static Mutex<Option<Pending>> {
    PENDING.get_or_init(|| Mutex::new(None))
}

/// Risk level of a mutating action, driving whether a confirmation popup is
/// shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    /// `!` in the README: requires confirmation when user handholding is on.
    Risky,
    /// `!!` in the README: requires confirmation when user handholding or
    /// user handhandholding is on.
    HighRisk,
}

/// Whether actions at `level` require a confirmation under `config`.
#[must_use]
pub fn requires_confirmation(config: &Config, level: RiskLevel) -> bool {
    match level {
        RiskLevel::Risky => config.user_handholding(),
        RiskLevel::HighRisk => config.user_handholding() || config.user_handhandholding(),
    }
}

/// Opens a confirmation popup; `yes` runs when the user confirms.
///
/// # Panics
///
/// Panics if the pending-confirmation lock is poisoned.
pub fn confirm(title: &str, lines: Vec<String>, yes: Pending) {
    // Opening a new confirmation replaces any pending one.
    *pending().lock().unwrap() = Some(yes);
    open_popup(title, &lines);
}

/// Runs the pending confirmation action and closes the popup. Exported to Lua
/// as `confirm_yes`.
pub fn confirm_yes(_: Object) {
    if let Some(yes) = pending().lock().unwrap().take() {
        close_popups();
        yes();
    }
}

/// Discards the pending confirmation and closes the popup. Exported to Lua as
/// `confirm_no`.
pub fn confirm_no(_: Object) {
    pending().lock().unwrap().take();
    close_popups();
}

/// Closes any confirmation popups that are still open.
fn close_popups() {
    let popups: Vec<Buffer> = api::list_bufs()
        .filter(|buffer| {
            let opts = OptionOpts::builder().buf(buffer.clone()).build();
            api::get_option_value::<String>("filetype", &opts)
                .is_ok_and(|filetype| filetype == CONFIRM_FILE_TYPE)
        })
        .collect();

    for popup in popups {
        let _ = api::command(&format!("bdelete {}", popup.handle()));
    }
}

/// Opens a floating window with `lines` as its content, `title` as its border
/// title and `y`/`n` keymaps wired to the pending confirmation.
fn open_popup(title: &str, lines: &[String]) {
    let in_buffer_list = false;
    let is_temporary = true;
    let mut buffer = match api::create_buf(in_buffer_list, is_temporary) {
        Ok(buffer) => buffer,
        Err(err) => {
            nvim_oxi::print!("failed to create confirmation buffer: {err}");
            return;
        }
    };

    let opts = OptionOpts::builder().buf(buffer.clone()).build();

    if let Err(err) = api::set_option_value("filetype", CONFIRM_FILE_TYPE, &opts) {
        nvim_oxi::print!("failed to set confirmation filetype: {err}");
        return;
    }

    if let Err(err) = api::set_option_value("modifiable", true, &opts) {
        nvim_oxi::print!("failed to set confirmation option value: {err}");
        return;
    }

    if let Err(err) = buffer.set_lines(0..0, true, lines.to_vec()) {
        nvim_oxi::print!("failed to write confirmation content: {err}");
        return;
    }

    if let Err(err) = api::set_option_value("modifiable", false, &opts) {
        nvim_oxi::print!("failed to set confirmation option value: {err}");
        return;
    }

    let keymap_opts = SetKeymapOpts::builder().silent(true).build();
    for (keys, lua) in [
        ("y", "confirm_yes"),
        ("<CR>", "confirm_yes"),
        ("n", "confirm_no"),
        ("q", "confirm_no"),
        ("<Esc>", "confirm_no"),
    ] {
        let command = format!(":lua require('mail_nvim').{lua}()<CR>");
        if let Err(err) = buffer.set_keymap(Mode::Normal, keys, &command, &keymap_opts) {
            nvim_oxi::print!("failed to set confirmation keymap: {err}");
            return;
        }
    }

    // Size the popup to its content and center it on screen.
    let global_opts = OptionOpts::builder().scope(OptionScope::Global).build();
    let columns =
        usize::try_from(api::get_option_value::<i64>("columns", &global_opts).unwrap_or(80))
            .unwrap_or(80);
    let screen_lines =
        usize::try_from(api::get_option_value::<i64>("lines", &global_opts).unwrap_or(24))
            .unwrap_or(24);

    let content_width = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);
    let width = (content_width + 2).clamp(10, columns.saturating_sub(2));
    let height = (lines.len() + 2).clamp(3, screen_lines.saturating_sub(2));

    let mut config = WindowConfig::builder();
    config
        .relative(WindowRelativeTo::Editor)
        .anchor(WindowAnchor::NorthWest)
        .row(u32::try_from(screen_lines.saturating_sub(height) / 2).unwrap_or(0))
        .col(u32::try_from(columns.saturating_sub(width) / 2).unwrap_or(0))
        .width(u32::try_from(width).unwrap_or(u32::MAX))
        .height(u32::try_from(height).unwrap_or(u32::MAX))
        .border(WindowBorder::Rounded)
        .title(WindowTitle::SimpleString(title.into()))
        .style(WindowStyle::Minimal);

    if let Err(err) = api::open_win(&buffer, true, &config.build()) {
        nvim_oxi::print!("failed to open confirmation popup: {err}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(handholding: Option<bool>, handhandholding: Option<bool>) -> Config {
        let mut builder = Config::builder();
        if let Some(value) = handholding {
            builder.user_handholding(value);
        }
        if let Some(value) = handhandholding {
            builder.user_handhandholding(value);
        }
        builder
            .build()
            .expect("expected default builder to be valid")
    }

    #[test]
    fn defaults_require_confirmation_for_both_levels() {
        let config = config(None, None);
        assert!(requires_confirmation(&config, RiskLevel::Risky));
        assert!(requires_confirmation(&config, RiskLevel::HighRisk));
    }

    #[test]
    fn disabling_handholding_disables_risky_actions() {
        let config = config(Some(false), Some(false));
        assert!(!requires_confirmation(&config, RiskLevel::Risky));
        assert!(!requires_confirmation(&config, RiskLevel::HighRisk));
    }

    #[test]
    fn handhandholding_alone_still_confirms_high_risk() {
        let config = config(Some(false), Some(true));
        assert!(!requires_confirmation(&config, RiskLevel::Risky));
        assert!(requires_confirmation(&config, RiskLevel::HighRisk));
    }

    #[test]
    fn handholding_alone_confirms_both_levels() {
        let config = config(Some(true), Some(false));
        assert!(requires_confirmation(&config, RiskLevel::Risky));
        assert!(requires_confirmation(&config, RiskLevel::HighRisk));
    }
}
