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
    utils::render::{ASYNC_RUNTIME, get_context, get_data, render},
};

use nvim_oxi::libuv::AsyncHandle;

pub struct AccountList;

impl UserCommand for AccountList {
    const NAME: Name = Name::new("MailAccountList");
    const DESCRIPTION: &'static str = "List all configured mail accounts";

    fn default_view_component() -> Option<UiViewComponent> {
        Some(UiViewComponent {
            id: "command-account-list".into(),
            name: "AccountList".into(),
            component_type: UiViewComponentType::Table,
            context: UiViewComponentContext {
                command_group: "Account".into(),
                command_type: "List".into(),
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

        let shared_data = Arc::new(Mutex::new(None));
        let shared_data_for_async = Arc::clone(&shared_data);

        let async_handle = AsyncHandle::new(move || {
            // This runs on the MAIN THREAD after .send() is called
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
                // Put data in the mailbox
                *shared_data_for_async.lock().unwrap() = Some(data);

                // 4. Ping Neovim to tell it "Data is ready!"
                let () = async_handle
                    .send()
                    .expect("failed to send async notification to Neovim");
            }
        });
    }
}
