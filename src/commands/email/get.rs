use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use nvim_oxi::libuv::AsyncHandle;

use crate::{
    api::{
        config::{
            Config,
            ui::view::{UiViewComponent, UiViewComponentContext, UiViewComponentType},
        },
        file::TryFile,
    },
    commands::prelude::*,
    utils::render::{ASYNC_RUNTIME, ComponentData, get_context, get_data, render},
};

pub struct EmailGet;

impl UserCommand for EmailGet {
    const NAME: Name = Name::new("MailEmail");
    const DESCRIPTION: &'static str = "Show the details to an e-mail";

    fn default_view_component() -> Option<UiViewComponent> {
        Some(UiViewComponent {
            id: "command-envelope-get".into(),
            name: "EmailGet".into(),
            component_type: UiViewComponentType::File,
            context: UiViewComponentContext {
                command_group: "Email".into(),
                command_type: "Get".into(),
                arguments: HashMap::new(),
                context: Vec::new(),
            },
            layout: None,
        })
    }

    fn callback(_: CommandArgs) {
        let current_buffer = api::get_current_buf();

        let config = Config::read_from_file(None).expect("failed to read config file");

        let mut view_component =
            Self::default_view_component().expect("expected default view component to be defined");

        let context = match get_context(Some(current_buffer), &view_component) {
            Ok(context) => context,
            Err(err) => bail!("failed to get context: {err}"),
        };

        view_component.context.context = context;

        let shared_component = Arc::new(Mutex::new(view_component.clone()));

        let shared_data = Arc::new(Mutex::<Option<ComponentData>>::new(None));
        let shared_data_for_async = Arc::clone(&shared_data);

        let async_handle = AsyncHandle::new(move || {
            let mut lock = shared_data.lock().unwrap();
            if let Some(data) = lock.take() {
                let component_for_schedule = Arc::clone(&shared_component);
                nvim_oxi::schedule(move |()| {
                    let component_info = component_for_schedule.lock().unwrap();
                    let _ = render(&component_info, data);
                });
            }
        })
        .expect("failed to create async handle");

        ASYNC_RUNTIME.spawn(async move {
            if let Ok(data) = get_data(&view_component, &config).await {
                *shared_data_for_async.lock().unwrap() = Some(data);

                let () = async_handle
                    .send()
                    .expect("failed to send async notification to Neovim");
            }
        });
    }
}
