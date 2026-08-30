//! Registry of the components currently rendered by a file-defined view.
//!
//! Each open pane is tracked by its [`UiViewComponent::id`] together with the
//! window and buffer it occupies. Enter actions and linked-pane previews use
//! this registry to target the right pane: `NewWindow`/`ReplaceView` need the
//! pane identity of the current component, and a linked list needs the
//! buffer/window of its follower (e.g. a reading pane).

use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

use nvim_oxi::api::{Buffer, Window};

use crate::api::config::ui::view::UiViewComponent;

/// A component pane opened by the view engine.
#[derive(Clone)]
pub(crate) struct ComponentInstance {
    /// The component definition currently rendered in the pane.
    pub component: UiViewComponent,
    /// The buffer the component is rendered into.
    pub buffer: Buffer,
    /// The window the pane occupies.
    pub window: Window,
}

/// The components of the open view, keyed by [`UiViewComponent::id`].
pub(crate) static VIEW_INSTANCES: LazyLock<RwLock<HashMap<String, ComponentInstance>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Registers (or replaces) the instance of the component with the given id.
pub(crate) fn register(id: &str, component: UiViewComponent, buffer: Buffer, window: Window) {
    VIEW_INSTANCES.write().unwrap().insert(
        id.to_string(),
        ComponentInstance {
            component,
            buffer,
            window,
        },
    );
}

/// Returns a copy of the instance registered for `id`.
#[must_use]
pub(crate) fn get(id: &str) -> Option<ComponentInstance> {
    VIEW_INSTANCES.read().unwrap().get(id).cloned()
}

/// Returns a copy of every registered pane.
#[must_use]
pub(crate) fn all() -> Vec<ComponentInstance> {
    VIEW_INSTANCES.read().unwrap().values().cloned().collect()
}

/// Re-registers `component` under `new_id`, dropping `old_id` when it still
/// points at the same pane (used when an enter action replaces a pane's
/// content).
pub(crate) fn replace(
    old_id: &str,
    new_id: &str,
    component: UiViewComponent,
    buffer: Buffer,
    window: Window,
) {
    let mut instances = VIEW_INSTANCES.write().unwrap();
    if instances
        .get(old_id)
        .is_some_and(|instance| instance.buffer == buffer)
    {
        instances.remove(old_id);
    }
    instances.insert(
        new_id.to_string(),
        ComponentInstance {
            component,
            buffer,
            window,
        },
    );
}

/// Clears the registry (used by tests between sessions).
pub(crate) fn clear() {
    VIEW_INSTANCES.write().unwrap().clear();
}
