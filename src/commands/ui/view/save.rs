//! The `MailUISave` command.

use std::fs;

use nvim_oxi::api::{self, Buffer};

use crate::api::config::ui::view::{UiView, UiViewComponent};
use crate::api::file::{TryFile, prepare_default_data_directory};
use crate::commands::prelude::*;
use crate::utils::buffer::metadata::BufferMetadata;
use crate::utils::buffer::render::FromBuffer;

pub struct Save;

impl UserCommand for Save {
    const NAME: Name = Name::new("MailUISave");
    const DESCRIPTION: &'static str = "Saves the current UI layout as a view";

    fn callback(args: CommandArgs) {
        let name = args
            .fargs
            .first()
            .map(String::as_str)
            .filter(|name| !name.is_empty())
            .unwrap_or("saved");
        let sanitized: String = name
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
            .collect();

        // Order the mail panes left-to-right and collect the component that is
        // rendered in each buffer.
        let mut windows: Vec<(usize, usize, Buffer)> = api::list_wins()
            .filter_map(|window| {
                let (row, col) = window.get_position().ok()?;
                let buffer = window.get_buf().ok()?;
                Some((col, row, buffer))
            })
            .collect();
        windows.sort_by_key(|(col, row, _)| (*col, *row));

        let components: Vec<UiViewComponent> = windows
            .into_iter()
            .filter_map(|(_, _, buffer)| {
                BufferMetadata::from_buffer(&buffer, None)
                    .ok()
                    .map(|metadata| metadata.component)
            })
            .collect();

        if components.is_empty() {
            bail!("no mail UI components are open");
        }

        let view = UiView {
            name: name.to_string(),
            components,
        };

        let dir = match prepare_default_data_directory() {
            Ok(dir) => dir.join("views"),
            Err(err) => bail!("failed to prepare data directory: {err}"),
        };

        if let Err(err) = fs::create_dir_all(&dir) {
            bail!("failed to create views directory: {err}");
        }

        let path = dir.join(format!("{sanitized}.json"));
        if let Err(err) = view.write_to_file(&path) {
            bail!("failed to write view file: {err}");
        }

        nvim::print!("saved view `{name}` to {}", path.display());
    }
}
