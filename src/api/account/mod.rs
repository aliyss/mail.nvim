pub mod commands;

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
