pub struct Account {
    pub name: String,
    pub backends: Option<String>,
    pub default: bool,
}

pub trait List<P> {
    /// Execute the list command using the provided mail provider.
    /// # Errors
    /// Returns an error if the command fails.
    fn execute(self, provider: &P) -> Result<Vec<Account>, String>;
}
