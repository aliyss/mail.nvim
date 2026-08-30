//! Buffer-local selection state for multi-select email actions.
//!
//! Selections are keyed by buffer handle so each open email list keeps its
//! own set of selected email ids, surviving in-place re-renders (refreshes,
//! pagination) since ids are stable across pages.

use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, RwLock};

use nvim_oxi::api::Buffer;

/// Selected email ids per buffer, keyed by buffer handle.
static SELECTION: LazyLock<RwLock<HashMap<i32, HashSet<String>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Marks `email_id` as selected in `buffer` (idempotent, unlike [`toggle`]).
///
/// Used when a whole range of rows is selected at once (e.g. a visual-mode
/// selection), where re-selecting an already-marked row must keep it marked.
///
/// # Panics
///
/// Panics if the selection lock is poisoned.
#[must_use]
pub fn select(buffer: &Buffer, email_id: &str) -> bool {
    select_by_handle(buffer.handle(), email_id)
}

/// Marks an id as selected without needing a live buffer (used by tests).
#[must_use]
pub(crate) fn select_by_handle(handle: i32, email_id: &str) -> bool {
    let mut map = SELECTION.write().unwrap();
    map.entry(handle).or_default().insert(email_id.to_string());
    true
}

/// Toggles whether `email_id` is selected in `buffer`; returns the new state.
///
/// # Panics
///
/// Panics if the selection lock is poisoned.
#[must_use]
pub fn toggle(buffer: &Buffer, email_id: &str) -> bool {
    toggle_by_handle(buffer.handle(), email_id)
}

/// Toggles a selection without needing a live buffer (used by tests).
#[must_use]
pub(crate) fn toggle_by_handle(handle: i32, email_id: &str) -> bool {
    let mut map = SELECTION.write().unwrap();
    let selected = map.entry(handle).or_default();
    if selected.remove(email_id) {
        false
    } else {
        selected.insert(email_id.to_string());
        true
    }
}

/// The email ids selected in `buffer`.
#[must_use]
pub fn selected_ids(buffer: &Buffer) -> HashSet<String> {
    selected_ids_by_handle(buffer.handle())
}

#[must_use]
pub(crate) fn selected_ids_by_handle(handle: i32) -> HashSet<String> {
    SELECTION
        .read()
        .unwrap()
        .get(&handle)
        .cloned()
        .unwrap_or_default()
}

/// Clears the selection of `buffer`.
pub fn clear(buffer: &Buffer) {
    clear_by_handle(buffer.handle());
}

pub(crate) fn clear_by_handle(handle: i32) {
    SELECTION.write().unwrap().remove(&handle);
}

#[cfg(test)]
mod tests {
    use super::*;

    // The tests share the global registry, so each uses distinct handles to
    // stay isolated under the default parallel test runner.
    #[test]
    fn toggle_adds_and_removes_ids() {
        assert!(toggle_by_handle(100, "10"));
        assert!(!toggle_by_handle(100, "10"));
        assert!(toggle_by_handle(100, "11"));

        assert_eq!(
            selected_ids_by_handle(100),
            HashSet::from(["11".to_string()])
        );
    }

    #[test]
    fn select_marks_an_id_idempotently() {
        assert!(select_by_handle(99, "10"));
        assert!(select_by_handle(99, "10"));
        assert_eq!(
            selected_ids_by_handle(99),
            HashSet::from(["10".to_string()])
        );
    }

    #[test]
    fn selection_is_per_buffer() {
        let _ = toggle_by_handle(101, "10");
        let _ = toggle_by_handle(102, "20");

        assert_eq!(
            selected_ids_by_handle(101),
            HashSet::from(["10".to_string()])
        );
        assert_eq!(
            selected_ids_by_handle(102),
            HashSet::from(["20".to_string()])
        );
    }

    #[test]
    fn clear_removes_everything() {
        let _ = toggle_by_handle(103, "10");
        clear_by_handle(103);
        assert!(selected_ids_by_handle(103).is_empty());
    }
}
