<div align="center">
    <h1>📫 mail.nvim</h1>
    <p>
        <strong>Neovim UI</strong> for checking your emails
    </p>
    <p>
        <img alt="TestedLinux" src="https://img.shields.io/badge/NixOS_25.11-blue?style=flat&logo=nixos&logoColor=white&label=linux&labelColor=gray&color=blue" />
        <img alt="TestedVim" src="https://img.shields.io/badge/NVIM_v0.11-green?style=flat&logo=neovim&logoColor=white&label=neovim&labelColor=gray&color=%23226b07" />
        <a href="http://discord.gg/zAypMTH">
            <img alt="SocialAlice" src="https://img.shields.io/badge/wonderland-green?style=flat&logo=discord&logoColor=white&label=support&labelColor=gray&color=%235765f2&link=http%3A%2F%2Fdiscord.gg%2FzAypMTH">
        </a>
    </p>
</div>

## Disclaimer
The idea is to adhere to [Zawinski's Law](https://en.m.wikipedia.org/wiki/Jamie_Zawinski), which states:

> Every program attempts to expand until it can read mail. Those programs which cannot so expand are replaced by ones which can.

Neovim is a great text editor, but it lacks a proper mail client. And to mitigate our common fear of having neovim replaced by another text editor (shall not be mentioned by name), We will do our best to make sure that never happens.

The project was made possible by the hard work layed out by:
- [pimalaya/himalaya](https://github.com/pimalaya/himalaya) - A clean mail client cli (still being used under the hood)
- [kristijanhusak/vim-dadbod-ui](https://github.com/kristijanhusak/vim-dadbod-ui) - A database UI for Neovim (for inspiration)
- [aliyss/vim-himalaya-ui](https://github.com/aliyss/vim-himalaya-ui) - The original plugin (which was itself inspired by the above two)

## Installation

### Prerequisites
- This plugin requires `Rust (Cargo)` to be installed

### Using [packer](https://github.com/wbthomason/packer.nvim)

```lua
use "https://github.com/aliyss/mail.nvim"
```

```vim
:PackerSync
```

### Using [vim-plug](https://github.com/junegunn/vim-plug)

```vim
Plug 'https://github.com/aliyss/mail.nvim'
```

```vim
:PlugInstall
```

### Using [lazy.nvim](https://github.com/folke/lazy.nvim)

```lua
return {
    "aliyss/mail.nvim",
    cmd = {
        "MailUI"
    },
    init = function()
        -- Your Mail Configuration
    end,
}
```

## Usage

If `himalaya` is not configured, the plugin will automatically prompt you with an installation wizard.

## Mail Commands

### Mail Command Definitions

**Flags:**
- `directory`: a directory path
- `view`: a view identifier
- `component`: a component identifier
- `tag`: a tag identifier
- `account`: a mail account identifier
- `folder`: a mail folder identifier
- `email`: an email identifier
- `thread`: an email thread identifier
- `template`: a mail template identifier
- `flag`: a mail flag identifier
- `command`: a shell command (shell escaped)
- `format`: a format identifier (plain, json, html, eml, etc.)
- `pagination`: pagination options (if pagination is disabled, all results are returned according to the limit set)
- `storage`: storage options (himalaya folder or local directory)

**Operators:**
- `+`: apply to all within the context
- `?`: optional
- `?:`: otherwise
- `:`: spec
- `!`: risk
- `!!`: high risk
- `=> ... =>`: if => then => else
- `(...)`: self-explanatory grouping
- `[]`: type array

**Abbreviations:**
- `tbd`: to be defined
- `bh`: buffer height
- `t`: true
- `f`: false

- `nmd`: mail.nvim directory

- `dv`: default view
- `cv`: current view

- `cc`: current component
- `sc`: saved component

- `da`: default account
- `ca`: current account
- `cda`: current else default account

- `df`: default folder ?? (dif)
- `dif`: default inbox folder
- `ddf`: default draft folder
- `dtf`: default trash folder
- `cf`: current folder
- `cdf`: current else default folder

- `ce`: current email
- `cet`: current email thread

- `dt`: default template
- `ct`: current template
- `st`: saved template
- `cdt`: current else default template

### Mail Config                     

All commands related to configuring mail.nvim and himalaya.

| MailConfig Commands                                            | Status | Description | Flags |
|---|:---:|---|---|
| `:MailConfig`                            | 🛠️ | Open the Mail Configuration Wizard                | |
| `:MailConfigFile`                        | 🛠️ | Open the Mail Configuration File                  | |
| `:MailConfigLocation`                    | ❌ | Set the location of the Mail Configuration        | dir |
| `:MailConfigHimalayaFile`                | ❌ | Open the Himalaya Configuration File              | |
| `:MailConfigHimalayaFileLocationSet`     | ❌ | Set the location of the Himalaya Configuration    | dir |
| `:MailConfigHimalayaFileLocationReset`   | ❌ | Set the location of the Himalaya Configuration    | |
| `:MailConfigEmailViewAsCommandSet`       | ❌ | Set the command to view email with a format       | format:(plain,html,...), command, capture_output?:t |
| `:MailConfigUserHandHoldingSwitchOn`     | ❌ | Risky actions require confirmation                | t/f |
| `:MailConfigUserHandHandHoldingSwitchOn` | ❌ | Risky Risky actions require confirmation          | t/f |


### Mail Help

All commands related to getting help about mail.nvim. As well as information about the project, contributing, support, etc.

| MailHelp Commands                                              | Status | Description | Flags |
|---|:---:|---|---|
| `:MailHelp`                              | ❌ | help                                              | |
| `:MailKeybindings`                       | ❌ | keybindings                                       | |
| `:MailAbout`                             | ❌ | about information                                 | |
| `:MailChangelog`                         | ❌ | changelog                                         | |
| `:MailLicense`                           | ❌ | license                                           | |
| `:MailContribute`                        | ❌ | contribute information                            | |
| `:MailSupport`                           | ❌ | support information                               | |
| `:MailIssueReport`                       | ❌ | Open the mail.nvim issue tracker                  | |
| `:MailFeatureRequest`                    | ❌ | Open the mail.nvim feature request tracker        | |


### Mail UI

All commands related to the Mail UI, views and components.

| MailUI Commands                                                | Status | Description | Flags |
|---|:---:|---|---|
| `:MailUI`                                | ❌ | Open the MailUI                                   | view?:dv |
| `:MailUIToggle`                          | ❌ | Toggle the MailUI                                 | (tags?, components?)?:cv |
| `:MailUIRefresh`                         | ❌ | Refresh the component contents                    | (tags?, components?)?:cc |
| `:MailUIClose`                           | ❌ | Close the MailUI                                  | |


| MailUIView Commands                                            | Status | Description | Flags |
|---|:---:|---|---|
| `:MailUIViewConfigFile`                  | ❌ | Open the view config file                         | view?:cv|
| `:MailUIViewList`                        | ❌ | List all saved views                              | |
| `:MailUIViewSave`                        | ❌ | Save the current view                             | tbd (open buffer positions, names...) |
| `:MailUIViewReset`                       | ❌ | Reset the current view in case layout changed     | |
| `:MailUIViewDelete`                      | ❌ | Delete a saved view                               | view?:cv!! |
| `:MailUIViewDefaultSet`                  | ❌ | Set the default view                              | view?:cv! |
| `:MailUIViewDefaultClear`                | ❌ | Reset the default view to the built-in one        | |


| MailUIViewComponent Commands                                   | Status | Description | Flags |
|---|:---:|---|---|
| `:MailUIViewComponentConfigFile`         | ❌ | Open the view config file to related line number  | component?:cc |
| `:MailUIViewComponentList`               | ❌ | List all components in a view                     | |
| `:MailUIViewComponentToggle`             | ❌ | Toggle a component in a view                      | component?:cc |
| `:MailUIViewComponentTypeSet`            | ❌ | Set the type of a component in a view             | component?:sc, view_type:(list/drawer/email/etc.) |
| `:MailUIViewComponentFeatureSet`         | ❌ | Set the feature of a component in a view          | component?:sc, feature:(...) |
| `:MailUIViewComponentTagAdd`             | ❌ | Add a tag to a component in a view                | component?:cc, tag |
| `:MailUIViewComponentTagRemove`          | ❌ | Remove the tag of a component in a view           | component?:cc, tag |
| `:MailUIViewComponentTagClear`           | ❌ | Clear all tags of a component in a view           | component?:cc |


### Mail Management

All commands related to managing mail accounts, folders, emails, threads, templates and tags.

| MailAccount Commands                                           | Status | Description | Flags |
|---|:---:|---|---|
| `:MailAccount`                           | 🛠️ | Show the details to the mail account              | account?:cda |
| `:MailAccountList`                       | 🛠️ | List all configured mail accounts                 | |
| `:MailAccountAdd`                        | ❌ | Add a mail account                                | tbd (defined by himalaya) |
| `:MailAccountEdit`                       | ❌ | Edit a mail account                               | account?:cda |
| `:MailAccountRemove`                     | ❌ | Remove a mail account                             | account?:ca!! |
| `:MailAccountDefaultSet`                 | ❌ | Set the default mail account                      | account?:ca! |


| MailFolder Commands                                            | Status | Description | Flags (+account?:cda) |
|---|:---:|---|---|
| `:MailFolder`                            | 🛠️ | Show the details to the mail folder               | folder?:cf |
| `:MailFolderList`                        | 🛠️ | List all folders in a mail account                | pagination?:t=>(page?:0, limit?:bh)=>limit? |
| `:MailFolderCreate`                      | 🛠️ | Create a mail folder                              | tbd (defined by himalaya) |
| `:MailFolderRename`                      | ❌ | Rename a mail folder                              | folder?:cf |
| `:MailFolderExpunge`                     | 🛠️ | Expunge a mail folder                             | folder?:cf!! |
| `:MailFolderPurge`                       | 🛠️ | Purge a mail folder                               | folder?:cf!! |
| `:MailFolderDelete`                      | 🛠️ | Delete a mail folder                              | folder?:cf!! |
| `:MailFolderDefaultInboxSet`             | ❌ | Set the default inbox folder                      | folder?:cf!|
| `:MailFolderDefaultInboxReset`           | ❌ | Reset the default inbox folder                    | folder?:cf! |
| `:MailFolderDefaultDraftSet`             | ❌ | Set the default draft folder                      | folder?:cf! |
| `:MailFolderDefaultDraftReset`           | ❌ | Reset the default draft folder                    | folder?:cf! |
| `:MailFolderDefaultTrashSet`             | ❌ | Set the default trash folder                      | folder?:cf! |
| `:MailFolderDefaultTrashReset`           | ❌ | Reset the default trash folder                    | folder?! |


| MailEmail Commands                                             | Status | Description | Flags (+account?:cda, +folder?:cdf)          |
|---|:---:|---|---|
| `:MailEmail`                             | ❌ | Show the details to an email                      | email?:ce, mark_read?:t |
| `:MailEmailList`                         | 🛠️ | List emails of a folder                           | pagination?:t=>(page?:0, limit?:bh)=>limit? |
| `:MailEmailCreate`                       | ❌ | Create a new email                                | tbd (defined by himalaya) |
| `:MailEmailSend`                         | ❌ | Send an email                                     | email?:ce! |
| `:MailEmailReply`                        | ❌ | Reply to an email                                 | email?:ce |
| `:MailEmailReplyAll`                     | ❌ | Reply All to an email                             | email?:ce |
| `:MailEmailForward`                      | ❌ | Forward an email                                  | email?:ce |
| `:MailEmailDiscard`                      | ❌ | Discard an email                                  | email?:ce! |
| `:MailEmailExport`                       | ❌ | Export an email                                   | email?:ce, dir?:nmd, format?:plain |
| `:MailEmailViewAs`                       | ❌ | View an email                                     | email?:ce, format?:plain, mark_read?:f |
| `:MailEmailSaveAsDraft`                  | ❌ | Save an email as draft                            | email?:ce, storage?:(folder?:ddf/dir?:nmd) |
| `:MailEmailSaveAsTemplate`               | ❌ | Save an email as template                         | email?:ce |
| `:MailEmailFlagAdd`                      | ❌ | Add a flag to an email                            | email?:ce, flag |
| `:MailEmailFlagRemove`                   | ❌ | Remove a flag from an email                       | email?:ce, flag |
| `:MailEmailFlagClear`                    | ❌ | Clear all flags from an email                     | email?:ce! |
| `:MailEmailToggleRead`                   | ❌ | Mark emails as read                               | email[]?:ce, mark_read?:(null/t/f) |
| `:MailEmailMove`                         | ❌ | Move emails to another folder                     | email[]?:ce, to_folder?:folder! |
| `:MailEmailCopy`                         | ❌ | Copy emails to another folder                     | email[]?:ce, to_folder?:folder! |
| `:MailEmailAttachmentsDownload`          | ❌ | Download attachments of emails                    | email[]?:ce, dir?:nmd |
| `:MailEmailDelete`                       | ❌ | Delete emails                                     | email[]?:ce!! |


| MailEmailThread Commands                                       | Status | Description | Flags (+account?:cda, +folder?:cdf, +email?:ce) |
|---|:---:|---|---|
| `:MailEmailThread`                       | ❌ | Show the details to a thread                      | thread?:cet, mark_read?:t |
| `:MailEmailThreadList`                   | ❌ | List threads of an email                          | pagination?:t=>(page?:0, limit?:bh)=>limit? |
| `:MailEmailThreadNext`                   | ❌ | Go to the next email in the thread                | thread?:cet |
| `:MailEmailThreadPrevious`               | ❌ | Go to the previous email in the thread            | thread?:cet |
| `:MailEmailThreadExport`                 | ❌ | Export emails in the thread                       | thread?:cet, dir?:nmd, format?:plain |
| `:MailEmailThreadMarkRead`               | ❌ | Mark emails in the thread as read                 | thread?:cet, ignore_emails[]? |
| `:MailEmailThreadMove`                   | ❌ | Move emails in the thread to another folder       | thread?:cet, to_folder?:folder |
| `:MailEmailThreadCopy`                   | ❌ | Copy emails in the thread to another folder       | thread?:cet, to_folder?:folder |
| `:MailEmailThreadAttachmentsDownload`    | ❌ | Download attachments of emails in the thread      | thread?:cet, dir?:nmd |


| MailTemplate Commands                                          | Status | Description | Flags (+account?:cda) |
|---|:---:|---|---|
| `:MailTemplate`                          | ❌ | Show the details to a mail template               | template?:cdt |
| `:MailTemplateList`                      | ❌ | List all mail templates                           | pagination?:t=>(page?:0, limit?:bh)=>limit? |
| `:MailTemplateCreate`                    | ❌ | Create a mail template                            | email?:ce?, tbd (defined by mail.nvim) |
| `:MailTemplateEdit`                      | ❌ | Edit a mail template                              | template?:cdt
| `:MailTemplateSave`                      | ❌ | Save a mail template                              | template?:ct, overwrite?:f |
| `:MailTemplateDelete`                    | ❌ | Delete a mail template                            | template?:ct! |
| `:MailTemplateDefaultSet`                | ❌ | Set the default mail template                     | template?:cst! |
| `:MailTemplateTypeSet`                   | ❌ | Set the tag of the mail template                  | template?:ct, type:(create/reply/forward) |


| MailTag Commands                                               | Status | Description | Flags (+account?:cda) |
|---|:---:|---|---|
| `:MailEmailFlag`                         | ❌ | Show the details to a mail flag                   | flag?:cf |
| `:MailEmailFlagList`                     | ❌ | List all mail tags                                | pagination?:t=>(page?:0, limit?:bh)=>limit? |
| `:MailEmailFlagCreate`                   | ❌ | Create a mail tag                                 | tbd (defined by himalaya) |
| `:MailEmailFlagEdit`                     | ❌ | Edit a mail tag                                   | flag?:cf |
| `:MailEmailFlagSave`                     | ❌ | Save a mail tag                                   | flag?:cf, overwrite?:f |
| `:MailEmailFlagDelete`                   | ❌ | Delete a mail tag                                 | flag?:cf!! |



## Additional Features

### Render Email HTML
When viewing an email, you can toggle between different views by using `:MailEmailViewAs`. This however requires you to have the different type of viewer installed. You can set any command you want to use to view an email by using the following command:
```vim
" Command format
:MailConfigEmailViewAsCommandSet <type> <command> [capture_output]

" Example
:MailConfigEmailViewAsCommandSet "html" "cha --type \"text/html\"" true
```

As you can see, I use `mhonarc` to convert the email to HTML and `chawan` to view it in the terminal. You can use any other tools of your choice.
Make sure to install them first though.

## Debugging
Run the command to build and run the plugin in one call

```bash
cargo build; cp ./target/debug/libmail_nvim.so ./lua/mail_nvim.so; nvim -c ":set rtp+=./" -c ":lua require(\"mail_nvim\")"
```
