//! This module contains the global help functions for the application.

/// Configuration for all settings within the Mailbox.
#[derive(Debug, Clone)]
pub struct HelpMessage;

impl HelpMessage {
    #[must_use]
    pub fn text() -> String {
        r"
mail.nvim HELP
==============

USAGE:
  :Mail [OPTIONS] [SUBCOMMAND]

OPTIONS:

SUBCOMMANDS:
  config              Manage configuration settings
  inbox               View your inbox
  send                Send an email
  read                Read an email
  delete              Delete an email
  search              Search emails
  help                Print this help information
"
        .trim()
        .to_owned()
    }
}
