use super::view::engine::open_ui_view;
use super::view::instances;
use super::{drawer_component, has_open_mail_buffer, setup_drawer_buffer};
use crate::api::config::Config;
use crate::api::config::ui::view::UiView;
use crate::api::file::TryFile;
use crate::commands::prelude::*;

use super::drawer::load_drawer;

pub struct Open;

impl UserCommand for Open {
    const NAME: Name = Name::new("MailUI");
    const DESCRIPTION: &'static str = "Opens the Mail UI";

    fn callback(_: CommandArgs) {
        if has_open_mail_buffer() {
            return; // The Mail UI is already open.
        }

        let config = match Config::read_from_file(None) {
            Ok(config) => config,
            Err(err) => bail!("failed to read config file: {err}"),
        };

        let view = match UiView::read_from_file(None) {
            Ok(view) => view,
            Err(err) => bail!("failed to read view file: {err}"),
        };

        if view.components.is_empty() {
            open_auto_created_view(config);
        } else if let Err(err) = open_ui_view(view, config) {
            bail!("failed to open UI view: {err}");
        }
    }
}

/// Opens the default view generated from the current configuration, falling
/// back to the classic drawer when no account is configured.
///
/// The auto-generated view (e.g. the Outlook-style layout) is persisted so
/// it can be customized and reused.
fn open_auto_created_view(config: Config) {
    let default = match UiView::regenerated_default() {
        Ok(default) => default,
        Err(err) => {
            nvim_oxi::print!("failed to create default view: {err:#}");
            open_drawer(config);
            return;
        }
    };

    if default.components.is_empty() {
        // No account configured yet: keep the classic drawer as the fallback.
        open_drawer(config);
        return;
    }

    if let Err(err) = default.write_default() {
        nvim_oxi::print!("failed to write default view: {err}");
    }

    if let Err(err) = open_ui_view(default, config) {
        nvim_oxi::print!("failed to open UI view: {err:#}");
    }
}

/// Opens the Mail UI drawer (the fallback when no view is configured).
fn open_drawer(config: Config) {
    let in_buffer_list = false;
    let is_temporary = true;
    let mut buffer = match api::create_buf(in_buffer_list, is_temporary) {
        Ok(buffer) => buffer,
        Err(err) => bail!("failed to create buffer: {err}"),
    };

    if let Err(err) = api::command("topleft vsplit") {
        bail!("failed to create a vertical split: {err}");
    }

    if let Err(err) = api::command("vertical resize 40") {
        bail!("failed to resize window: {err}");
    }

    if let Err(err) = api::set_current_buf(&buffer) {
        bail!("failed to set current buffer: {err}");
    }

    if let Err(err) = setup_drawer_buffer(&mut buffer) {
        bail!("failed to setup drawer buffer: {err}");
    }

    // Track the drawer pane so enter actions can target it.
    instances::register(
        &drawer_component().id,
        drawer_component(),
        buffer.clone(),
        api::get_current_win(),
    );

    // Load the accounts into the drawer asynchronously.
    load_drawer(buffer, config);
}
