//! Page navigation for the email list.

use nvim_oxi::api;

use crate::api::config::Config;
use crate::api::email::Email;
use crate::api::file::TryFile;
use crate::utils::buffer::metadata::BufferMetadata;
use crate::utils::buffer::render::FromBuffer;
use crate::utils::render::load_into;
use crate::utils::render::pagination::{apply_pagination, current_limit, current_page};
use crate::utils::render::table::render::Table;

/// Renders the next/previous page of the email list into the current buffer.
/// Exported to Lua as `email_list_page`.
pub fn email_list_page(arg: nvim_oxi::Object) {
    let delta: i64 = arg.try_into().unwrap_or_default();
    if delta == 0 {
        return;
    }

    let buffer = api::get_current_buf();

    let Ok(metadata) = BufferMetadata::from_buffer(&buffer, None) else {
        return;
    };

    if metadata.component.context.command_group != "Email"
        || (metadata.component.context.command_type != "List"
            && metadata.component.context.command_type != "Thread")
    {
        return;
    }

    let page = current_page(&metadata.component);
    let Some(limit) = current_limit(&metadata.component) else {
        return;
    };

    let new_page = i64::try_from(page)
        .unwrap_or(i64::MAX)
        .saturating_add(delta);

    if new_page < 1 {
        nvim_oxi::print!("Already on the first page.");
        return;
    }

    if delta > 0 {
        let rows = Table::<Vec<Email>>::from_buffer(&buffer, Some(metadata.line_count))
            .ok()
            .map_or(0, |table| table.data.len());

        if rows < limit {
            nvim_oxi::print!("No more emails.");
            return;
        }
    }

    let mut component = metadata.component;
    component
        .context
        .arguments
        .insert("page".into(), serde_json::json!(new_page));
    apply_pagination(&mut component);

    let config = match Config::read_from_file(None) {
        Ok(config) => config,
        Err(err) => {
            nvim_oxi::print!("failed to read config: {err}");
            return;
        }
    };

    load_into(component, config, buffer, None);
}
