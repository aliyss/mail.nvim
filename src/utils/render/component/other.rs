//! Fallback renderer for unknown component types.

use nvim_oxi::api::Buffer;

use crate::api::config::ui::view::UiViewComponent;
use crate::utils::buffer::metadata::BufferMetadata;
use crate::utils::render::ComponentData;
use crate::utils::render::component::Keymap;

/// # Errors
///
/// Returns an error if the fallback message fails to render into `buffer`.
pub fn render(
    component: &UiViewComponent,
    _data: ComponentData,
    buffer: &mut Buffer,
    metadata: &BufferMetadata,
) -> anyhow::Result<Vec<Keymap>> {
    let message = format!(
        "Rendering for component type {:?} is not implemented yet.",
        component.component_type
    );

    buffer.set_lines(
        metadata.line_count..metadata.line_count,
        true,
        vec![message],
    )?;

    Ok(Vec::new())
}
