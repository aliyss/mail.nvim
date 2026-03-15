pub mod commands;
use crate::utils::render::table::{RenderTable, RowBuilder};

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

    fn from_headers_and_rows(headers: Vec<String>, rows: Vec<RowBuilder>) -> Self {
        let mut accounts: Vec<Account> = Vec::new();
        let Some(name_index) = headers.iter().position(|h| h == "Name") else {
            return accounts;
        };
        let backend_index = headers.iter().position(|h| h == "Backend");
        let default_index = headers.iter().position(|h| h == "Default");
        for row in rows {
            let cells = row.cells;
            let name = match cells.get(name_index) {
                Some(cell) => cell.clone(),
                None => continue, // Skip rows without a name cell
            };

            let backend = if let Some(backend_index) = backend_index {
                cells.get(backend_index).cloned()
            } else {
                None
            };

            let default = if let Some(default_index) = default_index {
                cells
                    .get(default_index)
                    .is_some_and(|cell| cell.to_lowercase() == "yes")
            } else {
                false
            };

            accounts.push(Account::new(name, backend, default));
        }
        accounts
    }
}
