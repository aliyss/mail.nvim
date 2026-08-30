use pimalaya_tui::terminal::config::TomlConfig;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::HashMap, io};

use crate::api::config::Config;
use crate::api::file::TryFile;
use crate::providers::himalaya::HimalayaProvider;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiView {
    pub name: String,
    pub components: Vec<UiViewComponent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiViewComponent {
    pub id: String,
    pub name: String,
    pub component_type: UiViewComponentType,
    pub context: UiViewComponentContext,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<UiViewComponentLayout>,
    /// What pressing <CR> on a selected row does.
    ///
    /// When `None`, the action is inferred from the component type and its
    /// command (`[default_enter_action]`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_enter: Option<UiViewEnterAction>,
    /// The component pane that follows this one: selecting a row updates the
    /// linked pane without changing focus (e.g. a reading pane following an
    /// email list).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<UiViewComponentLink>,
}

impl UiViewComponent {
    /// The `<CR>` action of this component: the configured one or, when
    /// [`on_enter`](UiViewComponent::on_enter) is not set, the default
    /// inferred from the component type and command.
    #[must_use]
    pub fn enter_action(&self) -> UiViewEnterAction {
        self.on_enter
            .clone()
            .unwrap_or_else(|| self.default_enter_action())
    }

    /// Default `<CR>` behavior, inferred from the component type and command:
    ///
    /// * a list of emails (`Email` + `List`/`Thread`) opens the selected
    ///   email in a new window;
    /// * everything else (including the [`Drawer`](UiViewComponentType::Drawer),
    ///   which expands its tree in place) expands the selected row in place.
    #[must_use]
    pub fn default_enter_action(&self) -> UiViewEnterAction {
        match self.component_type {
            UiViewComponentType::List | UiViewComponentType::Table
                if self.context.command_group == "Email"
                    && matches!(self.context.command_type.as_str(), "List" | "Thread") =>
            {
                UiViewEnterAction::NewWindow
            }
            _ => UiViewEnterAction::ExpandView,
        }
    }
}

/// What happens when the user presses `<CR>` on a row of a component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiViewEnterAction {
    /// Expand the node under the cursor in place (drawer account -> folders).
    #[serde(rename = "expand_view")]
    ExpandView,
    /// Replace the current pane with the selected row's content.
    #[serde(rename = "replace_view")]
    ReplaceView,
    /// Open the selected row's content in a new window to the right.
    #[serde(rename = "new_window")]
    NewWindow,
}

/// Links a component to another pane that follows its selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiViewComponentLink {
    /// The [`UiViewComponent::id`] of the pane updated when the selection
    /// moves (e.g. a reading pane following an email list).
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiViewComponentContextContext {
    #[serde(rename = "account_id")]
    AccountId(String),
    #[serde(rename = "folder_id")]
    FolderId(String),
    #[serde(rename = "email_id")]
    EmailId(String),
}

impl UiViewComponentContextContext {
    #[must_use]
    pub fn to_id(id: &str, value: String) -> Option<Self> {
        if id == "account_id" {
            return Some(UiViewComponentContextContext::AccountId(value));
        } else if id == "folder_id" {
            return Some(UiViewComponentContextContext::FolderId(value));
        } else if id == "email_id" {
            return Some(UiViewComponentContextContext::EmailId(value));
        }
        None
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::AccountId(id) | Self::FolderId(id) | Self::EmailId(id) => id.as_str(),
        }
    }
    #[must_use]
    pub fn context_type(&self) -> &str {
        match self {
            Self::AccountId(_) => "account_id",
            Self::FolderId(_) => "folder_id",
            Self::EmailId(_) => "email_id",
        }
    }
}

impl From<&UiViewComponentContextContext> for String {
    fn from(context: &UiViewComponentContextContext) -> Self {
        match context {
            UiViewComponentContextContext::AccountId(id)
            | UiViewComponentContextContext::FolderId(id)
            | UiViewComponentContextContext::EmailId(id) => id.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiViewComponentContext {
    pub command_group: String,
    pub command_type: String,
    pub arguments: HashMap<String, Value>,
    pub context: Vec<UiViewComponentContextContext>,
}

impl UiViewComponentContext {
    pub fn get_required_context(
        &self,
        matcher: &str,
        error_msg: Option<&str>,
    ) -> anyhow::Result<&UiViewComponentContextContext> {
        for arg in &self.context {
            if matcher == arg.context_type() {
                return Ok(arg);
            }
        }

        Err(anyhow::anyhow!(
            "{}",
            error_msg.unwrap_or("required context argument not found")
        ))
    }

    #[must_use]
    pub fn get_optional_context(&self, matcher: &str) -> Option<&UiViewComponentContextContext> {
        self.context
            .iter()
            .find(|&arg| matcher == arg.context_type())
            .map(|v| v as _)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiViewComponentLayout {
    pub position: String,
    /// (horizontal, vertical)
    pub content_scrollable: (bool, bool),
    /// (x, y)
    pub location: (u32, u32),
    /// (width, height)
    pub size: (u32, Option<u32>),
    /// Whether size is a percentage of available space
    pub size_as_percentage: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiViewComponentType {
    /// A tree/sidebar (e.g. the account and folder drawer).
    #[serde(rename = "drawer")]
    Drawer,
    /// A table of rows (accounts, folders, emails, threads).
    #[serde(rename = "table")]
    Table,
    /// A list rendered as `Header: value` details.
    #[serde(rename = "detail")]
    Detail,
    /// A one-item preview of a list.
    #[serde(rename = "preview")]
    Preview,
    /// A full file view (e.g. an email message).
    #[serde(rename = "file")]
    File,
    /// A plain list (e.g. the middle pane of a mail client).
    #[serde(rename = "list")]
    List,
    /// A content pane (e.g. the reading pane of a mail client).
    #[serde(rename = "content")]
    Content,
    #[serde(rename = "other")]
    Other(String),
}

impl UiView {
    /// The default view for the current configuration: the Outlook-style
    /// three-pane layout for the default account, or a view without
    /// components (which makes [`open_ui_view`](crate::commands::ui::view::engine::open_ui_view)
    /// fall back to the classic drawer) when no account is configured.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration or the Himalaya configuration
    /// cannot be read.
    pub fn regenerated_default() -> Result<Self, io::Error> {
        let config = Config::read_from_file(None)?;
        let provider = HimalayaProvider::from_config(&config).map_err(io::Error::other)?;

        // The default account, or the first configured one when none is
        // marked as default.
        let account_id = provider
            .config()
            .get_default_account_config()
            .map(|(name, _)| name)
            .or_else(|| provider.config().accounts.keys().next().cloned())
            .unwrap_or_default();

        if account_id.is_empty() {
            Ok(UiView {
                name: "Default View".into(),
                components: Vec::new(),
            })
        } else {
            Ok(outlook_view(&account_id))
        }
    }

    /// Persists `self` to the default view file
    /// (`views/default.json` in the mail data directory).
    ///
    /// # Errors
    ///
    /// Returns an error if the data directory cannot be located or the file
    /// cannot be written.
    pub fn write_default(&self) -> Result<(), io::Error> {
        let path = crate::api::file::prepare_default_data_directory()?.join(Self::FILE_NAME);
        self.write_to_file(path)
    }
}

impl TryFile for UiView {
    type Error = io::Error;

    const FILE_NAME: &'static str = "views/default.json";

    fn try_default() -> Result<Self, Self::Error> {
        Self::regenerated_default()
    }
}

/// The Outlook-style three-pane layout described in the README: a folder
/// drawer on the left, the email list in the middle (linked to the reading
/// pane) and the reading pane on the right.
///
/// Used as the auto-created default view for `account_id`.
#[must_use]
pub fn outlook_view(account_id: &str) -> UiView {
    let layout = |position: &str, width: u32, size_as_percentage: bool| UiViewComponentLayout {
        position: position.into(),
        content_scrollable: (true, true),
        location: (0, 0),
        size: (width, None),
        size_as_percentage,
    };

    let inbox_context = vec![
        UiViewComponentContextContext::AccountId(account_id.to_string()),
        UiViewComponentContextContext::FolderId("INBOX".into()),
    ];

    UiView {
        name: "Outlook".into(),
        components: vec![
            UiViewComponent {
                id: "folders".into(),
                name: "Folders".into(),
                component_type: UiViewComponentType::Drawer,
                context: UiViewComponentContext {
                    command_group: "Account".into(),
                    command_type: "List".into(),
                    arguments: HashMap::new(),
                    context: Vec::new(),
                },
                layout: Some(layout("left", 30, true)),
                on_enter: None,
                link: None,
            },
            UiViewComponent {
                id: "emails".into(),
                name: "Emails".into(),
                component_type: UiViewComponentType::List,
                context: UiViewComponentContext {
                    command_group: "Email".into(),
                    command_type: "List".into(),
                    arguments: HashMap::from([("limit".into(), json!(50))]),
                    context: inbox_context.clone(),
                },
                layout: Some(layout("center", 50, true)),
                on_enter: Some(UiViewEnterAction::NewWindow),
                link: Some(UiViewComponentLink {
                    target: "reading".into(),
                }),
            },
            UiViewComponent {
                id: "reading".into(),
                name: "Reading Pane".into(),
                component_type: UiViewComponentType::Content,
                context: UiViewComponentContext {
                    command_group: "Email".into(),
                    command_type: "Get".into(),
                    arguments: HashMap::new(),
                    context: {
                        let mut context = inbox_context;
                        context.push(UiViewComponentContextContext::EmailId("1".into()));
                        context
                    },
                },
                layout: Some(layout("right", 0, false)),
                on_enter: None,
                link: None,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::PathBuf;

    #[test]
    fn create_drawer_config() {
        let mut arguments = HashMap::new();
        arguments.insert("limit".into(), json!(4));

        let context = vec![
            UiViewComponentContextContext::AccountId("nic@aliyssium.com".into()),
            UiViewComponentContextContext::FolderId("inbox".into()),
        ];

        let component = UiViewComponent {
            id: "accounts".into(),
            name: "Account List".into(),
            component_type: UiViewComponentType::Drawer,
            context: UiViewComponentContext {
                command_group: "MailEmail".into(),
                command_type: "List".into(),
                arguments,
                context,
            },
            layout: Some(UiViewComponentLayout {
                position: "left".into(),
                content_scrollable: (true, true),
                location: (0, 0),
                size: (30, Some(10)),
                size_as_percentage: true,
            }),
            on_enter: None,
            link: None,
        };
        assert_eq!(component.name, "Account List");
        assert_eq!(component.component_type, UiViewComponentType::Drawer);
        assert_eq!(component.context.command_group, "MailEmail");
        assert_eq!(component.context.command_type, "List");
        assert_eq!(component.context.arguments.get("limit"), Some(&json!(4)));
    }

    #[test]
    fn create_view_with_multiple_components() {
        let view = UiView {
            name: "Main View".into(),
            components: vec![
                UiViewComponent {
                    id: "drawer".into(),
                    name: "Drawer".into(),
                    component_type: UiViewComponentType::Drawer,
                    context: UiViewComponentContext {
                        command_group: "Mail".into(),
                        command_type: "Tree".into(),
                        arguments: HashMap::new(),
                        context: vec![],
                    },
                    layout: Some(UiViewComponentLayout {
                        position: "left".into(),
                        content_scrollable: (true, false),
                        location: (0, 0),
                        size: (30, None),
                        size_as_percentage: true,
                    }),
                    on_enter: None,
                    link: None,
                },
                UiViewComponent {
                    id: "table".into(),
                    name: "Table".into(),
                    component_type: UiViewComponentType::Table,
                    context: UiViewComponentContext {
                        command_group: "Mail".into(),
                        command_type: "List".into(),
                        arguments: HashMap::new(),
                        context: vec![],
                    },
                    layout: Some(UiViewComponentLayout {
                        position: "right".into(),
                        content_scrollable: (true, true),
                        location: (30, 0),
                        size: (70, None),
                        size_as_percentage: true,
                    }),
                    on_enter: None,
                    link: None,
                },
            ],
        };

        assert_eq!(view.components.len(), 2);
    }

    #[test]
    fn view_default_builder_like_behavior() {
        let view = UiView::try_default().expect("expected default UiView to be valid");

        if view.components.is_empty() {
            // No mail account configured: the classic drawer fallback.
            assert_eq!(view.name, "Default View");
        } else {
            // An account is configured: the Outlook-style default view.
            assert_eq!(view.name, "Outlook");
            assert_eq!(view.components.len(), 3);
        }
    }

    #[test]
    fn view_from_default_path() {
        // A fresh temp path, so the auto-creation (not a stale file) is
        // what gets exercised.
        let path = std::env::temp_dir()
            .join(format!("mail_nvim_view_test_{}", std::process::id()))
            .join(UiView::FILE_NAME);
        let _ = std::fs::remove_dir_all(path.parent().expect("temp path has a parent"));

        let view = UiView::read_from_file(Some(path))
            .expect("expected default view to be created automatically");

        if view.components.is_empty() {
            assert_eq!(view.name, "Default View");
        } else {
            assert_eq!(view.name, "Outlook");
            assert_eq!(view.components.len(), 3);
        }
    }

    #[test]
    fn view_from_invalid_path() {
        UiView::read_from_file(Some(PathBuf::from("/invalid/path/to/view.json")))
            .expect_err("expected hard-coded invalid path to fail");
    }

    fn component(
        id: &str,
        component_type: UiViewComponentType,
        command_group: &str,
        command_type: &str,
    ) -> UiViewComponent {
        UiViewComponent {
            id: id.into(),
            name: id.into(),
            component_type,
            context: UiViewComponentContext {
                command_group: command_group.into(),
                command_type: command_type.into(),
                arguments: HashMap::new(),
                context: Vec::new(),
            },
            layout: None,
            on_enter: None,
            link: None,
        }
    }

    #[test]
    fn drawer_defaults_to_expand_view() {
        let drawer = component("folders", UiViewComponentType::Drawer, "Account", "List");
        assert_eq!(drawer.enter_action(), UiViewEnterAction::ExpandView);
    }

    #[test]
    fn email_list_defaults_to_new_window() {
        let list = component("emails", UiViewComponentType::List, "Email", "List");
        assert_eq!(list.enter_action(), UiViewEnterAction::NewWindow);

        let thread = component("emails", UiViewComponentType::Table, "Email", "Thread");
        assert_eq!(thread.enter_action(), UiViewEnterAction::NewWindow);
    }

    #[test]
    fn other_lists_default_to_expand_view() {
        let account = component("accounts", UiViewComponentType::Table, "Account", "List");
        assert_eq!(account.enter_action(), UiViewEnterAction::ExpandView);

        let folders = component("folders", UiViewComponentType::Table, "Folder", "List");
        assert_eq!(folders.enter_action(), UiViewEnterAction::ExpandView);
    }

    #[test]
    fn explicit_on_enter_overrides_the_default() {
        let mut component = component("emails", UiViewComponentType::List, "Email", "List");
        component.on_enter = Some(UiViewEnterAction::ReplaceView);
        assert_eq!(component.enter_action(), UiViewEnterAction::ReplaceView);
    }

    #[test]
    fn view_round_trips_enter_action_and_link() {
        let mut component = component("emails", UiViewComponentType::List, "Email", "List");
        component.on_enter = Some(UiViewEnterAction::NewWindow);
        component.link = Some(UiViewComponentLink {
            target: "reading".into(),
        });

        let json = serde_json::to_string_pretty(&component).expect("component should serialize");
        let decoded: UiViewComponent =
            serde_json::from_str(&json).expect("component should round-trip");

        assert_eq!(decoded.on_enter, Some(UiViewEnterAction::NewWindow));
        assert_eq!(
            decoded.link,
            Some(UiViewComponentLink {
                target: "reading".into()
            })
        );
    }

    #[test]
    fn outlook_view_builds_the_three_panes() {
        let view = outlook_view("me@example.com");
        assert_eq!(view.name, "Outlook");
        assert_eq!(view.components.len(), 3);

        let [folders, emails, reading] = view.components.as_slice() else {
            panic!("expected exactly three components");
        };

        // The folder drawer on the left takes 30% of the space.
        assert_eq!(folders.id, "folders");
        assert_eq!(folders.component_type, UiViewComponentType::Drawer);
        assert_eq!(
            (
                folders.context.command_group.as_str(),
                folders.context.command_type.as_str()
            ),
            ("Account", "List")
        );
        assert!(folders.context.context.is_empty());
        assert_eq!(folders.layout.as_ref().expect("layout").size, (30, None));
        assert!(folders.layout.as_ref().expect("layout").size_as_percentage);

        // The email list in the middle takes 50% and drives the reading pane.
        assert_eq!(emails.id, "emails");
        assert_eq!(emails.component_type, UiViewComponentType::List);
        assert_eq!(
            (
                emails.context.command_group.as_str(),
                emails.context.command_type.as_str()
            ),
            ("Email", "List")
        );
        assert_eq!(emails.context.arguments.get("limit"), Some(&json!(50)));
        assert!(
            emails
                .context
                .context
                .contains(&UiViewComponentContextContext::AccountId(
                    "me@example.com".into()
                ))
        );
        assert!(
            emails
                .context
                .context
                .contains(&UiViewComponentContextContext::FolderId("INBOX".into()))
        );
        assert_eq!(emails.enter_action(), UiViewEnterAction::NewWindow);
        assert_eq!(
            emails.link.as_ref().map(|link| link.target.as_str()),
            Some("reading")
        );

        // The reading pane on the right fills whatever is left.
        assert_eq!(reading.id, "reading");
        assert_eq!(reading.component_type, UiViewComponentType::Content);
        assert_eq!(
            (
                reading.context.command_group.as_str(),
                reading.context.command_type.as_str()
            ),
            ("Email", "Get")
        );
        assert!(
            reading
                .context
                .context
                .contains(&UiViewComponentContextContext::EmailId("1".into()))
        );
        assert!(!reading.layout.as_ref().expect("layout").size_as_percentage);
    }

    #[test]
    fn view_parses_without_enter_action_and_link() {
        let json = serde_json::json!({
            "id": "emails",
            "name": "Emails",
            "component_type": "list",
            "context": {
                "command_group": "Email",
                "command_type": "List",
                "arguments": {},
                "context": [{ "account_id": "me@example.com" }, { "folder_id": "INBOX" }]
            }
        });

        let component: UiViewComponent =
            serde_json::from_value(json).expect("view without enter metadata should parse");

        assert_eq!(component.on_enter, None);
        assert_eq!(component.link, None);
        assert_eq!(component.enter_action(), UiViewEnterAction::NewWindow);
    }
}
