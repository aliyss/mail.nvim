use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    api::{
        config::{
            Config,
            ui::view::{UiViewComponent, UiViewComponentContext, UiViewComponentType},
        },
        file::TryFile,
    },
    commands::prelude::*,
    utils::render::{ASYNC_RUNTIME, get_context, get_data, new_async_handle, render, send_async},
};

pub struct FolderList;

impl UserCommand for FolderList {
    const NAME: Name = Name::new("MailFolderList");
    const DESCRIPTION: &'static str = "List all folders in a mail account";

    fn default_view_component() -> Option<UiViewComponent> {
        Some(UiViewComponent {
            id: "command-folder-list".into(),
            name: "FolderList".into(),
            component_type: UiViewComponentType::Table,
            context: UiViewComponentContext {
                command_group: "Folder".into(),
                command_type: "List".into(),
                arguments: HashMap::new(),
                context: Vec::new(),
            },
            layout: None,
            on_enter: None,
            link: None,
        })
    }

    fn callback(_: CommandArgs) {
        let current_buffer = api::get_current_buf();

        let Ok(config) = Config::read_from_file(None) else {
            bail!("failed to read config file");
        };

        let Some(mut view_component) = Self::default_view_component() else {
            bail!("expected default view component to be defined");
        };

        let context = match get_context(Some(current_buffer), &view_component) {
            Ok(context) => context,
            Err(err) => bail!("failed to get context: {err}"),
        };

        view_component.context.context = context;

        let shared_component = Arc::new(Mutex::new(view_component.clone()));

        let shared_data = Arc::new(Mutex::new(None));
        let shared_data_for_async = Arc::clone(&shared_data);

        let Some(async_handle) = new_async_handle(move || {
            let mut lock = shared_data.lock().unwrap();
            if let Some(data) = lock.take() {
                let component_for_schedule = Arc::clone(&shared_component);
                nvim_oxi::schedule(move |()| {
                    let component_info = component_for_schedule.lock().unwrap();
                    let _ = render(&component_info, data);
                });
            }
        }) else {
            return;
        };

        ASYNC_RUNTIME.spawn(async move {
            if let Ok(data) = get_data(&view_component, &config).await {
                *shared_data_for_async.lock().unwrap() = Some(data);
                send_async(&async_handle);
            }
        });
    }
}
