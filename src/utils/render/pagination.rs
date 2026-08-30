//! Pagination for list components.
//!
//! The amount of rows a list renders is always the line height of the window
//! it is shown in. The current page and the computed limit are stored in the
//! component's `arguments` so a refresh or a page flip keeps them.

use serde_json::Value;

use crate::api::config::ui::view::UiViewComponent;
use crate::api::email::arguments::EmailListArguments;

/// Reads the 1-based page number stored in the component's arguments.
#[must_use]
pub fn current_page(component: &UiViewComponent) -> usize {
    component
        .context
        .arguments
        .get("page")
        .and_then(Value::as_u64)
        .map_or(1, |page| usize::try_from(page.max(1)).unwrap_or(usize::MAX))
}

/// Reads the number of rows per page stored in the component's arguments.
#[must_use]
pub fn current_limit(component: &UiViewComponent) -> Option<usize> {
    component
        .context
        .arguments
        .get("limit")
        .and_then(Value::as_u64)
        .map(|limit| usize::try_from(limit.max(1)).unwrap_or(usize::MAX))
}

/// The height of the current window, in grid rows.
fn window_height() -> usize {
    nvim_oxi::api::get_current_win().get_height().unwrap_or(0) as usize
}

/// Estimates how many lines the component's metadata block will occupy once
/// it is rendered into a buffer.
fn metadata_line_count(component: &UiViewComponent) -> usize {
    // `+++` delimiters plus the pretty-printed component JSON.
    match serde_json::to_string_pretty(component) {
        Ok(json) => 2 + json.lines().count(),
        Err(_) => 0,
    }
}

/// Computes the number of rows that fill the current window, taking the
/// metadata block and the table header (header + separator line) into
/// account.
fn height_based_limit(component: &UiViewComponent) -> usize {
    let reserved = metadata_line_count(component) + 2;
    window_height().saturating_sub(reserved).max(1)
}

/// Stores the current page and a height-fitting limit in the component's
/// arguments so the email list always renders a full window.
pub fn apply_pagination(component: &mut UiViewComponent) {
    let page = current_page(component);

    // Insert placeholder values first so the serialized metadata already
    // contains both keys before the limit is computed. Replacing the numbers
    // afterwards does not change the amount of rendered lines.
    component
        .context
        .arguments
        .insert("page".into(), serde_json::json!(page));
    component
        .context
        .arguments
        .insert("limit".into(), serde_json::json!(1));

    let limit = height_based_limit(component);

    component
        .context
        .arguments
        .insert("limit".into(), serde_json::json!(limit));
}

/// Builds the email list arguments from the component's pagination settings.
#[must_use]
pub fn email_list_arguments(component: &UiViewComponent) -> Option<EmailListArguments> {
    let page = current_page(component);
    let limit = current_limit(component)?;
    Some(EmailListArguments::new(Some(page), Some(limit)))
}
