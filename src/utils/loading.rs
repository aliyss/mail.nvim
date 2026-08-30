//! Animated loading spinners.
//!
//! While an asynchronous operation is in flight (expanding an account,
//! opening a folder or an email, ...), the pane where the action was
//! triggered shows a turning braille spinner: on the drawer node that was
//! activated, or in the `Sel` column of the table row that was clicked. A
//! short-lived animation task advances the frames in place — only the
//! spinner cell is rewritten — so the rest of the pane stays untouched.
//!
//! The animation rewrites the spinner cell with `nvim_buf_set_text`, which
//! requires the buffer to be modifiable. `modifiable` is toggled exactly
//! twice per load — when the first spinner of a buffer is marked and when
//! its last spinner clears — never from the animation tick itself: setting
//! an option fires `OptionSet` autocommands, and doing that from inside a
//! libuv callback (which can run while nvim is already iterating
//! autocommands) aborts nvim.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

use nvim_oxi::api::opts::{OptionOpts, OptionScope};
use nvim_oxi::api::{self, Buffer};

use crate::utils::render::{ASYNC_RUNTIME, new_async_handle, send_async};

/// The braille spinner frames, cycled in place.
pub const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// How long each frame stays on screen.
const TICK: Duration = Duration::from_millis(100);

/// Identifies what is loading inside a buffer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Anchor {
    /// A table row, by its 0-based position among the rendered rows.
    Row(usize),
    /// A drawer account node, by account name.
    Account(String),
    /// A drawer action node.
    Action { account: String, folder_id: String },
}

/// The spinner state of one loading element.
#[derive(Debug)]
struct Spinner {
    buffer: Buffer,
    /// Index of the current frame in [`FRAMES`].
    frame: usize,
    /// Whether the renderer drew the spinner and recorded its position.
    drawn: bool,
    /// 0-based buffer line where the spinner is drawn.
    line: usize,
    /// Byte column where the spinner is drawn.
    column: usize,
    /// The text the spinner replaced (restored on clear).
    replaced: String,
}

/// The active spinners, keyed by `(buffer handle, anchor)` so several rows of
/// the same buffer can load at the same time.
static SPINNERS: LazyLock<Mutex<HashMap<(i32, Anchor), Spinner>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// The original `modifiable` value of each buffer with an active spinner,
/// recorded when its first spinner is marked and restored when the last one
/// clears.
static MODIFIABLE: LazyLock<Mutex<HashMap<i32, bool>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// Whether the animation task is currently running.
static ANIMATING: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));

/// Sets `modifiable` for `buffer` locally.
fn set_modifiable(buffer: &Buffer, value: bool) {
    let opts = OptionOpts::builder()
        .scope(OptionScope::Local)
        .buf(buffer.clone())
        .build();
    let _ = api::set_option_value("modifiable", value, &opts);
}

/// Starts a spinner for `anchor` in `buffer`.
///
/// The spinner is drawn by the renderer on the next render of the buffer
/// (which calls [`set_position`]); until then it is only registered. For the
/// first spinner of a buffer the buffer is made modifiable so the animation
/// can rewrite the spinner cell; the previous value is restored when the
/// last spinner clears (see [`clear`]).
///
/// Must be called on the main thread (a user action context), like every
/// nvim API call in this module.
///
/// # Panics
///
/// Panics if the spinner lock is poisoned.
pub fn mark(buffer: &Buffer, anchor: Anchor) {
    let handle = buffer.handle();
    SPINNERS.lock().unwrap().insert(
        (handle, anchor),
        Spinner {
            buffer: buffer.clone(),
            frame: 0,
            drawn: false,
            line: 0,
            column: 0,
            replaced: String::new(),
        },
    );

    let mut modifiable = MODIFIABLE.lock().unwrap();
    if !modifiable.contains_key(&handle) {
        let was: bool = api::get_option_value(
            "modifiable",
            &OptionOpts::builder()
                .scope(OptionScope::Local)
                .buf(buffer.clone())
                .build(),
        )
        .unwrap_or(true);
        modifiable.insert(handle, was);
        if !was {
            set_modifiable(buffer, true);
        }
    }

    ensure_animator();
}

/// Stops the spinner for `anchor`, restoring the text it replaced and, for
/// the last spinner of a buffer, the buffer's original `modifiable` value.
///
/// Must be called on the main thread (a `vim.schedule` callback), like every
/// nvim API call in this module.
///
/// # Panics
///
/// Panics if the spinner lock is poisoned.
pub fn clear(buffer: &Buffer, anchor: &Anchor) {
    let handle = buffer.handle();
    let spinner = SPINNERS
        .lock()
        .unwrap()
        .remove(&(handle, anchor.clone()));
    if let Some(spinner) = spinner {
        restore(&spinner);
    }

    let still_loading = SPINNERS
        .lock()
        .unwrap()
        .keys()
        .any(|(other, _)| *other == handle);
    if !still_loading {
        if let Some(was) = MODIFIABLE.lock().unwrap().remove(&handle) {
            if !was {
                set_modifiable(buffer, false);
            }
        }
    }
}

/// Whether `buffer` currently has at least one active spinner. Renderers use
/// this to keep the buffer editable while its spinner animates.
///
/// # Panics
///
/// Panics if the spinner lock is poisoned.
#[must_use]
pub fn is_active(buffer: &Buffer) -> bool {
    let handle = buffer.handle();
    SPINNERS.lock().unwrap().keys().any(|(other, _)| *other == handle)
}

/// Every spinner of `buffer`, for renderers to draw: the anchor together with
/// the current frame character.
///
/// # Panics
///
/// Panics if the spinner lock is poisoned.
#[must_use]
pub fn spinners(buffer: &Buffer) -> Vec<(Anchor, &'static str)> {
    SPINNERS
        .lock()
        .unwrap()
        .iter()
        .filter(|((handle, _), _)| *handle == buffer.handle())
        .map(|((_, anchor), spinner)| (anchor.clone(), FRAMES[spinner.frame]))
        .collect()
}

/// Records where the spinner of `anchor` was drawn: the 0-based buffer line
/// and byte column, plus the text the spinner replaced (restored on clear).
///
/// Renderers call this after writing their content so the animation can
/// update the cell in place.
///
/// # Panics
///
/// Panics if the spinner lock is poisoned.
pub fn set_position(
    buffer: &Buffer,
    anchor: &Anchor,
    line: usize,
    column: usize,
    replaced: String,
) {
    if let Some(spinner) = SPINNERS
        .lock()
        .unwrap()
        .get_mut(&(buffer.handle(), anchor.clone()))
    {
        spinner.drawn = true;
        spinner.line = line;
        spinner.column = column;
        spinner.replaced = replaced;
    }
}

/// A spinner that is cleared when dropped, so an in-flight load can borrow the
/// pane it was triggered from without manual cleanup on every path.
pub struct Guard {
    buffer: Buffer,
    anchor: Anchor,
}

impl Guard {
    /// Marks `anchor` in `buffer` and returns a guard that clears it on drop.
    ///
    /// # Panics
    ///
    /// Panics if the spinner lock is poisoned.
    #[must_use]
    pub fn new(buffer: Buffer, anchor: Anchor) -> Self {
        mark(&buffer, anchor.clone());
        Self { buffer, anchor }
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        clear(&self.buffer, &self.anchor);
    }
}

/// Starts the animation task if it is not running already. The task ticks
/// while at least one spinner exists and stops by itself once they are gone.
fn ensure_animator() {
    let mut animating = ANIMATING.lock().unwrap();
    if *animating {
        return;
    }
    *animating = true;
    drop(animating);

    let Some(handle) = new_async_handle(advance) else {
        *ANIMATING.lock().unwrap() = false;
        return;
    };

    ASYNC_RUNTIME.spawn(async move {
        loop {
            tokio::time::sleep(TICK).await;
            send_async(&handle);
            if !*ANIMATING.lock().unwrap() {
                break;
            }
        }
    });
}

/// Advances every spinner one frame, in place, and stops once nothing is
/// left to animate. Runs on the main thread via an [`AsyncHandle`](crate::utils::render::new_async_handle).
///
/// # Panics
///
/// Panics if the spinner or animation locks are poisoned.
pub(crate) fn advance() {
    let mut animating = ANIMATING.lock().unwrap();
    let mut spinners = SPINNERS.lock().unwrap();

    spinners.retain(|_, spinner| spinner.buffer.is_valid());

    if spinners.is_empty() {
        *animating = false;
        return;
    }

    // Forget the `modifiable` bookkeeping of buffers whose spinners vanished
    // with the buffer itself: there is nothing left to restore.
    MODIFIABLE.lock().unwrap().retain(|handle, _| {
        spinners.keys().any(|(spinner_handle, _)| spinner_handle == handle)
    });

    for spinner in spinners.values_mut() {
        if !spinner.drawn {
            continue;
        }
        spinner.frame = (spinner.frame + 1) % FRAMES.len();
        let frame = FRAMES[spinner.frame];
        // The buffer is modifiable because its spinner was marked (see
        // [`mark`]); a render while the spinner is active keeps it that way.
        // `nvim_buf_set_text` validates `end_row` as an *inclusive* line
        // index, so a single-line edit passes the same row twice
        // (`line..line`).
        let mut buffer = spinner.buffer.clone();
        let _ = buffer.set_text(
            spinner.line..spinner.line,
            spinner.column,
            spinner.column + frame.len(),
            [frame],
        );
    }
}

/// Writes `replaced` back over the spinner's cell.
///
/// Called from [`clear`], which restores the buffer's `modifiable` value
/// separately.
fn restore(spinner: &Spinner) {
    if !spinner.drawn || !spinner.buffer.is_valid() {
        return;
    }
    let frame_len = FRAMES[spinner.frame].len();
    // `nvim_buf_set_text` validates `end_row` as an *inclusive* line index,
    // so a single-line edit passes the same row twice (`line..line`).
    let mut buffer = spinner.buffer.clone();
    let _ = buffer.set_text(
        spinner.line..spinner.line,
        spinner.column,
        spinner.column + frame_len,
        [spinner.replaced.clone()],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_are_single_width_characters() {
        // The animation rewrites the spinner cell in place, so every frame
        // must occupy the same amount of terminal columns.
        for frame in FRAMES {
            assert_eq!(frame.chars().count(), 1, "frame {frame} should be one cell");
        }
        assert_eq!(FRAMES.len(), 10);
    }

    #[test]
    fn anchors_are_compared_by_value() {
        assert_eq!(Anchor::Row(1), Anchor::Row(1));
        assert_ne!(Anchor::Row(1), Anchor::Row(2));
        assert_eq!(
            Anchor::Action { account: "a".into(), folder_id: "b".into() },
            Anchor::Action { account: "a".into(), folder_id: "b".into() }
        );
    }
}
