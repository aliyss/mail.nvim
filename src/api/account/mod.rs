pub mod commands;
use crate::commands::ui::view::render::{RenderTable, RowBuilder};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    name: String,
    backend: Option<String>,
    default: bool,
}

impl Account {
    #[must_use]
    pub fn new(name: String, backend: Option<String>, default: bool) -> Self {
        Self {
            name,
            backend,
            default,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn backend(&self) -> Option<&str> {
        self.backend.as_deref()
    }

    #[must_use]
    pub fn is_default(&self) -> bool {
        self.default
    }
}

impl RenderTable for Vec<Account> {
    fn headers(&self) -> Vec<String> {
        vec![
            "Name".to_string(),
            "Backend".to_string(),
            "Default".to_string(),
        ]
    }

    fn rows(&self) -> Vec<RowBuilder> {
        self.iter()
            .map(|account| {
                RowBuilder::new()
                    .with_cell(account.name.clone())
                    .with_cell(
                        account
                            .backend
                            .clone()
                            .unwrap_or_else(|| "None".to_string()),
                    )
                    .with_cell(if account.default {
                        "Yes".to_string()
                    } else {
                        "No".to_string()
                    })
            })
            .collect()
    }
}
