//! Builds the window layout of a [`UiView`] from its components.
//!
//! Components are laid out left to right as vertical columns. The width of
//! each pane comes from its [`UiViewComponentLayout`] and can be expressed
//! either as a percentage of the remaining space or as a fixed number of
//! columns (`size_as_percentage`). The rightmost component always fills the
//! space that is left.

use nvim_oxi::api::opts::{OptionOpts, OptionScope};
use nvim_oxi::api::{self, Buffer};

use crate::api::config::Config;
use crate::api::config::ui::view::{UiView, UiViewComponent, UiViewComponentType};
use crate::commands::ui::drawer::load_drawer;
use crate::commands::ui::setup_drawer_buffer;
use crate::commands::ui::view::{instances, navigation};
use crate::utils::render::{create_base_buffer, last_rendered_width, load_into, render_into_buffer};

/// Resolves the width in columns each component should occupy, from left to
/// right. The last component is always `None` so it fills the remaining space.
///
/// A layout with `size_as_percentage` is relative to the space not yet taken
/// by the panes to its left; otherwise `size.0` is an absolute number of
/// columns. A component without a layout (or with `size.0 == 0`) splits the
/// remaining space in half, like a plain `vsplit`.
#[allow(clippy::cast_possible_truncation)] // widths always fit in `u32`.
fn compute_columns(view: &UiView, total: u32) -> Vec<Option<u32>> {
    let mut remaining = u64::from(total.max(1));
    let mut columns = Vec::with_capacity(view.components.len());

    for (index, component) in view.components.iter().enumerate() {
        let is_last = index + 1 == view.components.len();

        let width = component.layout.as_ref().and_then(|layout| {
            if layout.size.0 == 0 {
                return None; // "auto": use the default split width.
            }

            let desired = if layout.size_as_percentage {
                u64::from(layout.size.0) * remaining / 100
            } else {
                u64::from(layout.size.0)
            };

            let max = if is_last {
                remaining
            } else {
                remaining.saturating_sub(1).max(1)
            };

            Some(desired.min(max).max(1) as u32)
        });

        if is_last {
            columns.push(None);
        } else {
            match width {
                Some(width) => remaining -= u64::from(width),
                None => remaining /= 2,
            }
            columns.push(width);
        }
    }

    columns
}

/// Creates the windows and buffers for a file-defined view, laying components
/// out left to right as vertical columns.
///
/// Returns the created buffers alongside their component so callers can load
/// the data for each one. This is separated from [`open_ui_view`] so the
/// layout logic can be tested without a provider or a config.
///
/// # Errors
///
/// Returns an error if a buffer, window or split cannot be created.
#[allow(clippy::needless_pass_by_value)] // `view` is owned for ergonomics.
pub fn create_view_layout(view: UiView) -> anyhow::Result<Vec<(UiViewComponent, Buffer)>> {
    let components = view.components.clone();
    if components.is_empty() {
        return Ok(Vec::new());
    }

    let total = api::get_current_win().get_width().unwrap_or_default();

    // Create one column per component: the original window becomes the leftmost
    // pane and each `rightbelow vsplit` carves a new pane out of it.
    let mut buffers = Vec::with_capacity(components.len());
    let mut windows = Vec::with_capacity(components.len());

    for (index, component) in components.iter().enumerate() {
        if index > 0 {
            api::command("rightbelow vsplit")?;
        }

        let opts = OptionOpts::builder().scope(OptionScope::Local).build();
        let mut buffer = create_base_buffer(&opts)?;

        if component.component_type == UiViewComponentType::Drawer {
            setup_drawer_buffer(&mut buffer)?;
        }

        let window = api::get_current_win();

        // Track the pane so enter actions and linked previews can target it.
        instances::register(
            &component.id,
            component.clone(),
            buffer.clone(),
            window.clone(),
        );

        // Live preview: update the linked pane when the cursor moves.
        if component.link.is_some() {
            navigation::bind_linked_preview(&buffer)?;
        }

        windows.push(window);
        buffers.push((component.clone(), buffer));
    }

    // Then size each pane from left to right: resizing a pane steals from the
    // windows to its right, so the rightmost pane keeps whatever is left.
    let columns = compute_columns(&view, total);

    for (index, width) in columns.into_iter().flatten().enumerate() {
        api::set_current_win(&windows[index])?;
        api::command(&format!("vertical resize {width}"))?;
    }

    Ok(buffers)
}

/// Opens a file-defined view: creates the window layout and asynchronously
/// loads each component into its own buffer.
///
/// # Errors
///
/// Returns an error if a buffer, window or split cannot be created.
#[allow(clippy::needless_pass_by_value)] // `view` is owned for ergonomics.
pub fn open_ui_view(view: UiView, config: Config) -> anyhow::Result<()> {
    let buffers = create_view_layout(view)?;

    for (component, buffer) in buffers {
        if component.component_type == UiViewComponentType::Drawer {
            load_drawer(buffer, config.clone());
        } else {
            load_into(component, config.clone(), buffer, None);
        }
    }

    Ok(())
}

/// Exported to Lua as `recalculate_layout`. Bound to `WinNew`, `WinClosed`,
/// `WinEnter` and `WinResized` so the pane sizes of an open view keep
/// matching their component layouts whenever a window changes.
pub fn recalculate_layout(_: nvim_oxi::Object) {
    recalculate_view();
}

/// Recomputes the width of every open pane from its component's layout and
/// reapplies it, left to right, re-rendering the panes whose width changed.
///
/// Used when a window is added (opening an email), removed (closing one) or
/// resized, so e.g. an email list keeps its configured percentage instead of
/// keeping a stale width. The last pane still fills whatever is left (see
/// [`compute_columns`]); panes without a layout default to sharing the
/// remaining space, like a plain `vsplit`.
fn recalculate_view() {
    let Some(panes) = open_panes() else {
        return;
    };

    // Lay the panes out left to right; with a single pane there is nothing
    // to lay out, but the pane still has to be re-rendered below when its
    // window grew (e.g. after a split closed) so its content keeps filling
    // the new width.
    if panes.len() > 1 {
        let total: u32 = panes
            .iter()
            .map(|pane| pane.window.get_width().unwrap_or_default())
            .sum();
        if total == 0 {
            return;
        }

        let view = UiView {
            name: "_layout".into(),
            components: panes.iter().map(|pane| pane.component.clone()).collect(),
        };

        for (index, width) in compute_columns(&view, total).into_iter().enumerate() {
            let Some(width) = width else {
                continue;
            };
            let mut window = panes[index].window.clone();
            let _ = window.set_width(width);
        }
    }

    // Rebuild the content of every pane whose window width no longer matches
    // the width it was rendered at: tables and messages have widths baked in
    // at render time, so a pane that changed width (resized directly, or the
    // last one expanded when a split closed) has to be re-rendered from the
    // cached data to fill its window again.
    for pane in &panes {
        if pane.window.is_valid()
            && pane.window.get_width().unwrap_or_default() != last_rendered_width(&pane.buffer)
        {
            rerender_pane(pane);
        }
    }
}

/// The registered panes that still have a valid window, ordered left to
/// right by their position on screen.
fn open_panes() -> Option<Vec<instances::ComponentInstance>> {
    let mut panes = instances::all();
    panes.retain(|pane| pane.window.is_valid() && pane.buffer.is_valid());
    if panes.is_empty() {
        return None;
    }

    panes.sort_by_key(|pane| pane.window.get_position().map_or(0, |(_, column)| column));
    Some(panes)
}

/// Re-renders the registered pane of `buffer` from its cached data at the
/// window's current width. Used when a new window steals width from the pane
/// (e.g. an email list before an email opens to its right), which the width
/// comparison in [`recalculate_view`] cannot detect for panes without an
/// explicit layout.
pub(crate) fn rerender_pane_by_buffer(buffer: &Buffer) {
    if let Some(pane) = instances::all()
        .into_iter()
        .find(|pane| pane.buffer == *buffer)
    {
        rerender_pane(&pane);
    }
}

/// Re-renders `pane` from its cached data at the window's current width,
/// restoring the user's window, buffer and cursor afterwards.
fn rerender_pane(pane: &instances::ComponentInstance) {
    let Some(data) = crate::utils::render::cached_pane_data(&pane.buffer) else {
        return;
    };

    let mut current_window = api::get_current_win();
    let current_buffer = api::get_current_buf();
    let current_cursor = current_window.get_cursor().unwrap_or((1, 0));

    if api::set_current_win(&pane.window).is_err() {
        return;
    }

    let mut buffer = pane.buffer.clone();
    if let Err(err) = render_into_buffer(&mut buffer, &pane.component, data) {
        nvim_oxi::print!("failed to re-render pane: {err}");
    }

    let _ = api::set_current_win(&current_window);
    let _ = api::set_current_buf(&current_buffer);
    if current_window.is_valid() {
        let rows = buffer.line_count().unwrap_or(1);
        let row = current_cursor.0.min(rows).max(1);
        let _ = current_window.set_cursor(row, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout(size: (u32, Option<u32>), size_as_percentage: bool) -> UiViewComponentStub {
        UiViewComponentStub {
            size,
            size_as_percentage,
        }
    }

    // A minimal stand-in for the config types so the tests do not depend on
    // constructing the full component context.
    struct UiViewComponentStub {
        size: (u32, Option<u32>),
        size_as_percentage: bool,
    }

    fn widths(columns: &[Option<u32>]) -> Vec<Option<u32>> {
        columns.to_vec()
    }

    fn build_view(layouts: &[Option<UiViewComponentStub>]) -> UiView {
        let components = layouts
            .iter()
            .map(|stub| crate::api::config::ui::view::UiViewComponent {
                id: "id".into(),
                name: "comp".into(),
                component_type: UiViewComponentType::Table,
                context: crate::api::config::ui::view::UiViewComponentContext {
                    command_group: "Email".into(),
                    command_type: "List".into(),
                    arguments: Default::default(),
                    context: Vec::new(),
                },
                layout: stub.as_ref().map(|s| {
                    crate::api::config::ui::view::UiViewComponentLayout {
                        position: "left".into(),
                        content_scrollable: (true, true),
                        location: (0, 0),
                        size: s.size,
                        size_as_percentage: s.size_as_percentage,
                    }
                }),
                on_enter: None,
                link: None,
            })
            .collect();

        UiView {
            name: "test".into(),
            components,
        }
    }

    #[test]
    fn single_component_fills_the_width() {
        let view = build_view(&[Some(layout((40, None), true))]);
        assert_eq!(widths(&compute_columns(&view, 100)), vec![None]);
    }

    #[test]
    fn percentages_are_relative_to_the_remaining_space() {
        let view = build_view(&[
            Some(layout((25, None), true)),
            Some(layout((50, None), true)),
            None,
        ]);

        // 100 columns: drawer 25, then 50% of the remaining 75, rest last.
        assert_eq!(
            widths(&compute_columns(&view, 100)),
            vec![Some(25), Some(37), None]
        );
    }

    #[test]
    fn fixed_columns_leave_the_rest_to_the_last_component() {
        let view = build_view(&[
            Some(layout((30, None), false)),
            Some(layout((40, None), false)),
            None,
        ]);

        assert_eq!(
            widths(&compute_columns(&view, 100)),
            vec![Some(30), Some(40), None]
        );
    }

    #[test]
    fn widths_are_clamped_to_the_available_space() {
        let view = build_view(&[Some(layout((90, None), false)), None]);

        // A 90-col drawer leaves a tiny (1-col) list.
        assert_eq!(widths(&compute_columns(&view, 100)), vec![Some(90), None]);
    }

    #[test]
    fn empty_view_produces_no_columns() {
        let view = build_view(&[]);
        assert!(compute_columns(&view, 80).is_empty());
    }
}
