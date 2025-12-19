use crate::api::{
    account::commands::{GetAccount, ListAccounts},
    folder::commands::GetFolder,
};

pub mod himalaya;

pub trait Provider: GetAccount + GetFolder + ListAccounts {}
impl<T: GetAccount + GetFolder + ListAccounts> Provider for T {}
