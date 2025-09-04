/// Configuration for all settings within the Mailbox.
#[derive(Debug, Clone)]
pub struct Help {}

impl Help {
    pub fn help() -> String {
        let help_text = r#"
Help mail.nvim
=======================

Usage:
  :Mail [OPTIONS] [SUBCOMMAND]
Options:

Subcommands:
  config              Manage configuration settings
  inbox               View your inbox
  send                Send an email
  read                Read an email
  delete              Delete an email
  search              Search emails
  help                Print this help information
"#;
        help_text.to_string()
    }
}
