use crate::api::account::Account;
use crate::api::folder::commands::ListFolders;
pub mod table;

use self::table::Table;
use std::path::PathBuf;
use std::sync::LazyLock;

use anyhow::Context;
use nvim_oxi::api::opts::{OptionOpts, OptionScope};
use nvim_oxi::api::{self, Buffer};

use crate::api::account::commands::{GetAccount, ListAccounts};
use crate::api::config::Config;
use crate::api::config::ui::view::{UiView, UiViewComponent, UiViewComponentType};
use crate::api::file::TryFile;
use crate::utils::buffer::name::{get_command_buffer_metadata, set_command_buffer_metadata};
use tokio::runtime::Runtime;

pub static ASYNC_RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime")
});

type RenderResult<T> = anyhow::Result<T>;

pub(crate) async fn render_ui_view_from_config(buffer: &mut Buffer, config_path: Option<PathBuf>) {
    if let Err(err) = render_ui_view_from_config_inner(buffer, config_path).await {
        eprintln!("Error rendering UI view: {err:#}");
    }
}

async fn render_ui_view_from_config_inner(
    buffer: &mut Buffer,
    config_path: Option<PathBuf>,
) -> RenderResult<()> {
    let config = Config::read_from_file(config_path).context("failed to read config file")?;

    let file_name = Some(&config.default_view_path).map(|path| {
        let actual_path = PathBuf::from(path);
        if actual_path.is_absolute() {
            actual_path
        } else {
            let folder_path = "views/";
            PathBuf::from(folder_path).join(actual_path)
        }
    });

    let opts = OptionOpts::builder().scope(OptionScope::Local).build();
    api::set_option_value("modifiable", true, &opts)?;

    buffer.set_lines(.., false, ["Rendering Mail UI View...", ""])?;

    let view = UiView::read_from_file_with_path(file_name)
        .context("failed to read UiView from specified file or default")?;

    buffer.set_lines(.., false, ["Rendering Mail UI View...", &view.name])?;

    let json_view_string =
        serde_json::to_string_pretty(&view).context("failed to serialize UiView to JSON string")?;
    buffer.set_lines(.., false, [json_view_string])?;

    for component in view.components {
        render_ui_view_from_component(buffer.clone(), None, component, config.clone()).await;
    }

    api::set_option_value("modifiable", false, &opts)?;

    Ok(())
}

pub(crate) async fn render_ui_view_from_component(
    mut buffer: Buffer,
    current_buffer: Option<Buffer>,
    component: UiViewComponent,
    config: Config,
) {
    // For now, other types are synchronous or unimplemented stubs
    let opts = OptionOpts::builder().scope(OptionScope::Local).build();
    let _ = api::set_option_value("modifiable", true, &opts);

    match component.component_type {
        UiViewComponentType::Drawer => render_drawer(&component),
        UiViewComponentType::Detail => render_detail(&component),
        UiViewComponentType::Preview => render_preview(&component),
        UiViewComponentType::File => render_file(&component),
        UiViewComponentType::Table => {
            render_table_async(&component, config.clone(), &mut buffer, current_buffer)
                .await
                .expect("failed to render table component");
        }
        UiViewComponentType::Other(_) => render_other(&component),
    }

    set_command_buffer_metadata(&mut buffer, &component, &config);
    let _ = api::set_option_value("modifiable", false, &opts);
}

async fn render_table_async(
    component: &UiViewComponent,
    config: Config,
    buffer: &mut Buffer,
    current_buffer: Option<Buffer>,
) -> anyhow::Result<()> {
    let provider = match config.to_provider() {
        Ok(provider) => provider,
        Err(e) => {
            api::err_writeln(&format!("Failed to initialize provider: {e}"));
            anyhow::bail!("failed to initialize provider: {e}");
        }
    };
    match component.context.command_group.as_str() {
        "Account" => {
            if component.context.command_type == "List" {
                let accounts = provider
                    .list_accounts()
                    .context("failed to list accounts")?;
                let table = Table::new(accounts);
                table.render(buffer)?;
            }
        }
        "Folder" => {
            if component.context.command_type == "List" {
                let account_data =
                    fetch_account_context(component, &config.clone(), current_buffer)?;

                if let Some(account) = account_data {
                    let mut buffer_clone = buffer.clone();
                    let folders_result = provider.list_folders(&account).await;

                    match folders_result {
                        Ok(folders) => {
                            let table = Table::new(folders);
                            // Ensure the buffer is still valid and modifiable here
                            let _ = table.render(&mut buffer_clone);
                        }
                        Err(e) => {
                            api::err_writeln(&format!("Failed to list folders: {e}"));
                        }
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}

fn fetch_account_context(
    component: &UiViewComponent,
    config: &Config,
    current_buffer: Option<Buffer>,
) -> anyhow::Result<Option<Account>> {
    if let Some(account_id) = component
        .context
        .context
        .get("account")
        .and_then(|v| v.as_str())
    {
        let provider = config.to_provider().context("provider init failed")?;
        return Ok(provider.get_account(account_id).unwrap_or(None));
    }

    if let Some(mut buf) = current_buffer {
        // Metadata access involves Neovim API calls, must be scheduled
        let config = config.clone();

        let metadata = match get_command_buffer_metadata(&mut buf, &config) {
            Ok(meta) => meta,
            Err(e) => {
                println!("Failed to get buffer metadata: {e:?}");
                return Ok(None);
            }
        };

        let has_table_data = match Table::<Vec<Account>>::from_buffer_lines(&metadata, &mut buf) {
            Ok(data) => data,
            Err(e) => {
                println!("Failed to parse table data from buffer: {e:?}");
                return Ok(None);
            }
        };

        let table_data = has_table_data.unwrap_or_default();

        let (row, _) = match buf.get_mark('\"') {
            Ok(pos) => pos,
            Err(e) => {
                println!("Failed to get cursor position: {e}");
                return Ok(None);
            }
        };

        let buffer_line_count = match buf.line_count() {
            Ok(count) => count,
            Err(e) => {
                println!("Failed to get buffer line count: {e}");
                return Ok(None);
            }
        };

        let table_data_len = table_data.len();

        if row == 0 || row > buffer_line_count {
            println!("Cursor position is out of buffer bounds");
            return Ok(None);
        }

        let result = table_data
            .get(row - (1 + buffer_line_count - table_data_len))
            .cloned();
        return Ok(result);
    }

    Ok(None)
}

fn render_drawer(_component: &UiViewComponent) {
    println!("Drawer rendering not implemented yet");
}

fn render_detail(_component: &UiViewComponent) {
    println!("Detail rendering not implemented yet");
}

fn render_preview(_component: &UiViewComponent) {
    println!("Preview rendering not implemented yet");
}

fn render_file(_component: &UiViewComponent) {
    println!("File rendering not implemented yet");
}

fn render_other(_component: &UiViewComponent) {
    println!("Other component rendering not implemented yet");
}
