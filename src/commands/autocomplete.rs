use std::collections::HashMap;
use std::sync::LazyLock;

static EMPTY: LazyLock<Commands> = LazyLock::new(Commands::empty);

#[derive(Debug, Clone, Default)]
pub struct Commands {
    pub children: HashMap<&'static str, Commands>,
}

impl FromIterator<(&'static str, Commands)> for Commands {
    fn from_iter<T: IntoIterator<Item = (&'static str, Commands)>>(iter: T) -> Self {
        let mut root = Self::default();

        for (key, children) in iter {
            root.children.insert(key, children);
        }

        root
    }
}

impl Commands {
    pub fn empty() -> Self {
        Self::default()
    }
}

pub struct Autocomplete {
    commands: Commands,
}

impl Autocomplete {
    pub fn new(commands: Commands) -> Self {
        Self { commands }
    }

    pub fn complete(&self, input: &str) -> Vec<String> {
        let last_token = input.split_whitespace().last().unwrap_or_default();
        let ends_with_whitespace = input.ends_with(' ');

        let (commands, prefix) = input
            .split_whitespace()
            .skip(1)
            .try_fold((&self.commands, None::<&str>), |(tree, _), token| {
                let is_last_token = token == last_token && !ends_with_whitespace;

                match tree.children.get(token) {
                    // Matched an intermediate subcommand; continue processing.
                    Some(child) if !is_last_token => Ok((child, None)),
                    // User has committed this token, even though it doesn't map to any commands.
                    _ if ends_with_whitespace => Err((&*EMPTY, None)),
                    // Provide autocomplete suggestions.
                    _ => Err((tree, Some(token))),
                }
            })
            .unwrap_or_else(|state| state);

        let prefix = prefix.unwrap_or("");
        let mut suggestions = commands
            .children
            .keys()
            .filter(|k| k.starts_with(prefix))
            .map(|k| (**k).to_string())
            .collect::<Vec<String>>();

        suggestions.sort();
        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::commands::command_tree::command_tree;

    // TODO: Create a Mock command tree so the tests are not relying on real data that can change.
    static AUTOCOMPLETE: LazyLock<Autocomplete> =
        LazyLock::new(|| Autocomplete::new(command_tree()));

    #[test]
    fn level_2() {
        let input = "Mail ";
        let mut suggestions = AUTOCOMPLETE.complete(input);
        let mut expected = vec![
            "account".to_owned(),
            "config".to_owned(),
            "email".to_owned(),
            "folder".to_owned(),
            "help".to_owned(),
            "tag".to_owned(),
            "template".to_owned(),
            "ui".to_owned(),
        ];

        suggestions.sort();
        expected.sort();

        assert_eq!(suggestions, expected);
    }

    #[test]
    fn level_3() {
        let input = "Mail help ";
        let mut suggestions = AUTOCOMPLETE.complete(input);
        let mut expected = vec![
            "about".to_owned(),
            "changelog".to_owned(),
            "contribute".to_owned(),
            "feature-request".to_owned(),
            "issue-report".to_owned(),
            "keybindings".to_owned(),
            "license".to_owned(),
            "support".to_owned(),
        ];

        suggestions.sort();
        expected.sort();

        assert_eq!(suggestions, expected);
    }

    #[test]
    fn exact_command() {
        let input = "Mail help";
        let mut suggestions = AUTOCOMPLETE.complete(input);
        let mut expected = vec!["help".to_owned()];

        suggestions.sort();
        expected.sort();

        assert_eq!(suggestions, expected);
    }

    #[test]
    fn multiple_suggestions() {
        let input = "Mail help c";
        let mut suggestions = AUTOCOMPLETE.complete(input);
        let mut expected = vec!["changelog".to_owned(), "contribute".to_owned()];

        suggestions.sort();
        expected.sort();

        assert_eq!(suggestions, expected);
    }

    #[test]
    fn valid_prefix_but_with_space() {
        let input = "Mail accou ";
        let mut suggestions = AUTOCOMPLETE.complete(input);
        let mut expected = Vec::<String>::new();

        suggestions.sort();
        expected.sort();

        assert_eq!(suggestions, expected);
    }
}
