use derive_builder::Builder;
use serde::{Deserialize, Serialize};

/// Marker traits
pub trait CommandGroup {}
pub trait CommandType<CG: CommandGroup> {}
pub trait CommandArguments<CG: CommandGroup, CT: CommandType<CG>> {}
pub trait CommandContext<CG: CommandGroup, CT: CommandType<CG>> {}

/// Represents the ui view component context configuration.
#[derive(Debug, Clone, PartialEq, Eq, Builder, Serialize, Deserialize)]
#[builder(setter(strip_option))]
pub struct UiViewComponentContext<CG, CT, CA, CC>
where
    CG: CommandGroup,
    CT: CommandType<CG>,
    CA: CommandArguments<CG, CT>,
    CC: CommandContext<CG, CT>,
{
    #[builder(setter(into))]
    command_group: CG,

    #[builder(setter(into))]
    command_type: CT,

    #[builder(setter(into))]
    arguments: CA,

    #[builder(setter(into))]
    context: CC,
}

/// Represents the ui view component layout configuration.
#[derive(Debug, Clone, PartialEq, Eq, Builder, Serialize, Deserialize)]
#[builder(setter(strip_option))]
pub struct UiViewComponentLayout {
    #[builder(setter(into))]
    position: String,

    // (horizontal, vertical)
    #[builder(setter(into))]
    content_scrollable: (bool, bool),

    // (x, y)
    #[builder(setter(into))]
    location: (u32, u32),

    // (width, height)
    #[builder(setter(into))]
    size: (u32, Option<u32>),

    // Whether the size is a percentage of the total available space.
    #[builder(setter(into))]
    size_as_percentage: bool,
}

/// Represents the ui view component type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiViewComponentType {
    Table,
    Drawer,
    Detail,
    Preview,
    File,
    Other(String),
}

/// Represents the ui view component configuration.
#[derive(Debug, Clone, PartialEq, Eq, Builder, Serialize, Deserialize)]
#[builder(setter(strip_option))]
pub struct UiViewComponent<CG, CT, CA, CC>
where
    CG: CommandGroup,
    CT: CommandType<CG>,
    CA: CommandArguments<CG, CT>,
    CC: CommandContext<CG, CT>,
{
    #[builder(setter(into))]
    id: String,

    #[builder(setter(into))]
    name: String,

    #[builder(setter(into))]
    component_type: UiViewComponentType,

    #[builder(setter(into))]
    context: UiViewComponentContext<CG, CT, CA, CC>,

    #[builder(setter(into))]
    layout: UiViewComponentLayout,
}

#[derive(Debug, Clone, PartialEq, Eq, Builder, Serialize, Deserialize)]
#[builder(setter(strip_option))]
pub struct UiView<CG, CT, CA, CC>
where
    CG: CommandGroup,
    CT: CommandType<CG>,
    CA: CommandArguments<CG, CT>,
    CC: CommandContext<CG, CT>,
{
    #[builder(setter(into))]
    name: String,
    #[builder(setter(into), default)]
    components: Vec<UiViewComponent<CG, CT, CA, CC>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCommandGroup;
    impl CommandGroup for TestCommandGroup {}
    struct TestCommandType;
    impl CommandType<TestCommandGroup> for TestCommandType {}
    struct TestCommandArguments {
        account_name: Option<String>,
        backend: Option<String>,
        default: Option<bool>,
    }
    impl CommandArguments<TestCommandGroup, TestCommandType> for TestCommandArguments {}
    struct TestCommandContext {
        user_id: Option<String>,
    }
    impl CommandContext<TestCommandGroup, TestCommandType> for TestCommandContext {}

    #[test]
    fn make_test_view() {
        let component_context = UiViewComponentContext {
            command_group: TestCommandGroup,
            command_type: TestCommandType,
            arguments: TestCommandArguments {
                account_name: None,
                backend: None,
                default: None,
            },
            context: TestCommandContext { user_id: None },
        };
        let component_layout = UiViewComponentLayout {
            position: "left".into(),
            content_scrollable: (true, true),
            location: (0, 0),
            size: (30, None),
            size_as_percentage: true,
        };
        let component = UiViewComponent {
            id: "accounts".into(),
            name: "Account List".into(),
            component_type: UiViewComponentType::Drawer,
            context: component_context,
            layout: component_layout,
        };
        let _view =
            UiView::<TestCommandGroup, TestCommandType, TestCommandArguments, TestCommandContext> {
                name: "Main View".into(),
                components: vec![component],
            };
    }
}
