use crate::api::account::Account;
use crate::api::email::Email;
use crate::api::email::commands::ListEmails;
use crate::api::folder::Folder;
use crate::api::folder::commands::ListFolders;
use crate::utils::buffer::render::{FromBuffer, ToBuffer};
use crate::utils::keymaps::create_localized_keymap;
use crate::utils::render::table::context::fetch_row_from_buffer;
use crate::utils::render::table::render::Table;

pub mod table;

use std::sync::LazyLock;

use anyhow::Context;
use nvim_oxi::Object;
use nvim_oxi::api::opts::{OptionOpts, OptionScope, SetKeymapOpts};
use nvim_oxi::api::types::Mode;
use nvim_oxi::api::{self, Buffer};

use crate::api::account::commands::ListAccounts;
use crate::api::config::Config;
use crate::api::config::ui::view::{
    UiViewComponent, UiViewComponentContextContext, UiViewComponentType,
};
use crate::utils::buffer::metadata::BufferMetadata;
use tokio::runtime::Runtime;

pub static ASYNC_RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime")
});

pub enum ComponentData {
    Accounts(Vec<Account>),
    Folders(Vec<Folder>),
    Emails(Vec<Email>),
    None,
}

fn get_optional_context_by_id<'a>(
    matcher: &str,
    component: &'a UiViewComponent,
    metadata: Option<&'a BufferMetadata>,
) -> Option<&'a UiViewComponentContextContext> {
    if let Some(ctx) = component.context.get_optional_context(matcher) {
        return Some(ctx);
    }

    if let Some(buffer_metadata) = metadata {
        return get_optional_context_by_id(matcher, &buffer_metadata.component, None);
    }

    None
}

fn get_required_context_by_id<'a>(
    matcher: &str,
    component: &'a UiViewComponent,
    metadata: Option<&'a BufferMetadata>,
) -> anyhow::Result<&'a UiViewComponentContextContext> {
    match component.context.get_required_context(matcher, None) {
        Ok(ctx) => Ok(ctx),
        Err(err) => {
            if let Some(buffer_metadata) = metadata {
                return get_required_context_by_id(matcher, &buffer_metadata.component, None);
            }
            anyhow::bail!("required context not found: {err:#}");
        }
    }
}

pub fn get_context(
    current_buffer: Option<Buffer>,
    component: &UiViewComponent,
) -> anyhow::Result<Vec<UiViewComponentContextContext>> {
    let mut context: Vec<UiViewComponentContextContext> = Vec::new();

    if component.context.command_group.as_str() == "Folder"
        && component.context.command_type == "List"
    {
        let buffer_metadata = current_buffer
            .as_ref()
            .and_then(|buf| BufferMetadata::from_buffer(buf, None).ok());

        let account_id =
            get_optional_context_by_id("account_id", component, buffer_metadata.as_ref());

        if let Some(account_id) = account_id {
            context.push(account_id.clone());
            return Ok(context);
        }

        if let Some(buffer) = current_buffer {
            let row = match fetch_row_from_buffer::<Vec<Account>>(
                &buffer,
                buffer_metadata.map_or(0, |meta| meta.line_count),
            ) {
                Ok(row) => row,
                Err(_err) => {
                    return Ok(context);
                }
            };

            context.push(UiViewComponentContextContext::AccountId(
                row.name().to_string(),
            ));
        }
    } else if component.context.command_group.as_str() == "Email"
        && component.context.command_type == "List"
    {
        let buffer_metadata = current_buffer
            .as_ref()
            .and_then(|buf| BufferMetadata::from_buffer(buf, None).ok());

        let account_id =
            get_optional_context_by_id("account_id", component, buffer_metadata.as_ref());

        let folder_id =
            get_optional_context_by_id("folder_id", component, buffer_metadata.as_ref());

        if let Some(account_id) = account_id {
            context.push(account_id.clone());
        }

        if let Some(folder_id) = folder_id {
            context.push(folder_id.clone());
        }

        if let Some(_) = folder_id
            && let Some(_) = account_id
        {
            return Ok(context);
        }

        if let Some(buffer) = current_buffer
            && let Some(buffer_metadata) = buffer_metadata
        {
            if buffer_metadata.component.context.command_group.as_str() == "Account" {
                let row = match fetch_row_from_buffer::<Vec<Account>>(
                    &buffer,
                    buffer_metadata.line_count,
                ) {
                    Ok(row) => row,
                    Err(_err) => {
                        return Ok(context);
                    }
                };

                context.push(UiViewComponentContextContext::AccountId(
                    row.name().to_string(),
                ));
            } else if buffer_metadata.component.context.command_group.as_str() == "Folder" {
                let row =
                    match fetch_row_from_buffer::<Vec<Folder>>(&buffer, buffer_metadata.line_count)
                    {
                        Ok(row) => row,
                        Err(_err) => {
                            return Ok(context);
                        }
                    };

                context.push(UiViewComponentContextContext::FolderId(
                    row.id().to_string(),
                ));
            }
        }
    }

    Ok(context)
}

pub async fn get_data(
    component: &UiViewComponent,
    config: &Config,
) -> anyhow::Result<ComponentData> {
    let provider = config
        .to_provider()
        .context("failed to initialize provider")?;

    match component.context.command_group.as_str() {
        "Account" => {
            if component.context.command_type == "List" {
                let accounts = provider
                    .list_accounts()
                    .context("failed to list accounts")?;
                return Ok(ComponentData::Accounts(accounts));
            }
        }
        "Folder" => {
            if component.context.command_type == "List" {
                let account_id = component.context.get_required_context("account_id", None)?;

                let folders = provider
                    .list_folders(account_id.as_str())
                    .await
                    .context("failed to list folders")?;

                return Ok(ComponentData::Folders(folders));
            }
        }
        "Email" => {
            if component.context.command_type == "List" {
                let account_id = component.context.get_required_context("account_id", None)?;
                let folder_id = component.context.get_optional_context("folder_id");

                let emails = match provider
                    .list_emails(
                        account_id.as_str(),
                        folder_id.map(UiViewComponentContextContext::as_str),
                        None,
                    )
                    .await
                {
                    Ok(emails) => emails,
                    Err(_err) => {
                        anyhow::bail!("failed to list emails.");
                    }
                };

                return Ok(ComponentData::Emails(emails));
            }
        }
        _ => {}
    }

    Ok(ComponentData::None)
}

pub fn create_base_buffer(opts: &OptionOpts) -> anyhow::Result<Buffer> {
    let in_buffer_list = false;
    let is_temporary = true;
    let buffer = match api::create_buf(in_buffer_list, is_temporary) {
        Ok(buffer) => buffer,
        Err(err) => anyhow::bail!("failed to create buffer: {err}"),
    };

    if let Err(err) = api::set_current_buf(&buffer) {
        anyhow::bail!("failed to set current buffer: {err}");
    }

    let options: [(&'static str, Object); 3] = [
        // Allows users to use `ftplugin` to customize the buffer.
        ("filetype", Object::from("mail-table")),
        // Prevents users from saving the file.
        ("buftype", Object::from("nofile")),
        // Prevents users from entering INSERT mode.
        ("modifiable", Object::from(true)),
    ];

    for (name, value) in options {
        if let Err(err) = api::set_option_value(name, value, opts) {
            anyhow::bail!("failed to set option value: {err}");
        }
    }

    Ok(buffer)
}

pub fn render(component: &UiViewComponent, data: ComponentData) -> anyhow::Result<()> {
    let opts = OptionOpts::builder().scope(OptionScope::Local).build();
    let mut buffer = create_base_buffer(&opts)?;

    let metadata = match BufferMetadata::new(component.clone()).to_buffer(&mut buffer, 0) {
        Ok(metadata) => metadata,
        Err(err) => anyhow::bail!("failed to render buffer metadata: {err}"),
    };

    let mut keymaps = Vec::from([(Mode::Normal, "q", ":bdelete<CR>".to_string())]);

    match component.component_type {
        UiViewComponentType::Drawer => render_drawer(component),
        UiViewComponentType::Detail => render_detail(component),
        UiViewComponentType::Preview => render_preview(component),
        UiViewComponentType::File => render_file(component),
        UiViewComponentType::Table => match data {
            ComponentData::Accounts(accounts) => {
                let table = match Table::<Vec<Account>>::new(accounts)
                    .to_buffer(&mut buffer, metadata.line_count)
                {
                    Ok(table) => table,
                    Err(err) => anyhow::bail!("failed to render accounts table: {err}"),
                };

                let start_line = metadata.line_count + table.offset + 1;
                let end_line = start_line + table.data.len();
                let localized_keymap = create_localized_keymap(
                    "MailFolderList",
                    start_line,
                    end_line,
                    "No account selected",
                );

                keymaps.push((Mode::Normal, "i", localized_keymap.clone()));
                keymaps.push((Mode::Normal, "<CR>", localized_keymap));
            }
            ComponentData::Folders(folders) => {
                let table = match Table::<Vec<Folder>>::new(folders)
                    .to_buffer(&mut buffer, metadata.line_count)
                {
                    Ok(table) => table,
                    Err(err) => anyhow::bail!("failed to render folders table: {err}"),
                };

                let start_line = metadata.line_count + table.offset + 1;
                let end_line = start_line + table.data.len();
                let localized_keymap = create_localized_keymap(
                    "MailEmailList",
                    start_line,
                    end_line,
                    "No folder selected",
                );

                keymaps.push((Mode::Normal, "i", localized_keymap.clone()));
                keymaps.push((Mode::Normal, "<CR>", localized_keymap));
            }
            ComponentData::Emails(emails) => {
                let table = Table::<Vec<Email>>::new(emails);
                table.to_buffer(&mut buffer, metadata.line_count)?;
            }
            ComponentData::None => {
                nvim_oxi::print!("None rendering not implemented yet.");
            }
        },
        UiViewComponentType::Other(_) => render_other(component),
    }

    let keymap_opts = SetKeymapOpts::builder().silent(true).build();

    for (mode, keys, command) in keymaps {
        if let Err(err) = buffer.set_keymap(mode, keys, &command, &keymap_opts) {
            anyhow::bail!("failed to set keymap: {err}");
        }
    }

    api::set_option_value("modifiable", false, &opts)?;

    Ok(())
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
