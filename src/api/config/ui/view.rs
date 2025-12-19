use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, io};

use crate::api::file::TryFile;

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
    pub layout: UiViewComponentLayout,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiViewComponentContext {
    pub command_group: String,
    pub command_type: String,
    pub arguments: HashMap<String, Value>,
    pub context: HashMap<String, Value>,
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
    Table,
    Drawer,
    Detail,
    Preview,
    File,
    Other(String),
}

impl TryFile for UiView {
    type Error = io::Error;

    const FILE_NAME: &'static str = "views/default.json";

    fn try_default() -> Result<Self, Self::Error> {
        // Minimal but valid default view
        Ok(UiView {
            name: "Default View".into(),
            components: Vec::new(),
        })
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

        let mut context = HashMap::new();
        context.insert("account".into(), json!("nic@aliyssium.com"));
        context.insert("folder".into(), json!("inbox"));

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
            layout: UiViewComponentLayout {
                position: "left".into(),
                content_scrollable: (true, true),
                location: (0, 0),
                size: (30, Some(10)),
                size_as_percentage: true,
            },
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
                        context: HashMap::new(),
                    },
                    layout: UiViewComponentLayout {
                        position: "left".into(),
                        content_scrollable: (true, false),
                        location: (0, 0),
                        size: (30, None),
                        size_as_percentage: true,
                    },
                },
                UiViewComponent {
                    id: "table".into(),
                    name: "Table".into(),
                    component_type: UiViewComponentType::Table,
                    context: UiViewComponentContext {
                        command_group: "Mail".into(),
                        command_type: "List".into(),
                        arguments: HashMap::new(),
                        context: HashMap::new(),
                    },
                    layout: UiViewComponentLayout {
                        position: "right".into(),
                        content_scrollable: (true, true),
                        location: (30, 0),
                        size: (70, None),
                        size_as_percentage: true,
                    },
                },
            ],
        };

        assert_eq!(view.components.len(), 2);
    }

    #[test]
    fn view_default_builder_like_behavior() {
        let view = UiView::try_default().expect("expected default UiView to be valid");

        assert_eq!(view.name, "Default View");
        assert!(view.components.is_empty());
    }

    #[test]
    fn view_from_default_path() {
        let view = UiView::read_from_file(None)
            .expect("expected default view to be created automatically");

        assert_eq!(view.name, "Default View");
    }

    #[test]
    fn view_from_invalid_path() {
        UiView::read_from_file(Some(PathBuf::from("/invalid/path/to/view.json")))
            .expect_err("expected hard-coded invalid path to fail");
    }
}
