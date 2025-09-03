#![allow(clippy::module_inception)]

//! Global Config API
//!
//! These are the global configuration options for the application.

mod config;
mod config_email;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewAsFormat {
    Plain,
    Json,
    HTML,
}
