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
    commands::{completion, prelude::*},
    utils::render::{
        ASYNC_RUNTIME, ComponentData, get_context, get_data, new_async_handle, render, send_async,
    },
};

pub struct EmailGet;

impl UserCommand for EmailGet {
    const NAME: Name = Name::new("MailEmail");
    const DESCRIPTION: &'static str = "Show the details to an e-mail";

    fn complete(arg_lead: &str, cmd_line: &str, _cursor_pos: usize) -> Vec<String> {
        // Email ids of the account/folder in the current buffer (or of the
        // account named on the command line), as fetched so far.
        let (account, folder) = completion::current_context();
        let account = completion::account_from(cmd_line).or(account);
        let Some(account) = account else {
            return Vec::new();
        };
        completion::filter(arg_lead, completion::email_ids(&account, folder.as_deref()))
    }

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

        let shared_data = Arc::new(Mutex::<Option<ComponentData>>::new(None));
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
