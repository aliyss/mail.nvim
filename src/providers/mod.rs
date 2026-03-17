use crate::api::{
    account::commands::{GetAccount, ListAccounts},
    email::commands::ListEmails,
    folder::commands::{CreateFolder, DeleteFolder, GetFolder, ListFolders},
};

pub mod himalaya;

pub trait Provider:
    GetAccount
    + GetFolder
    + ListAccounts
    + ListFolders
    + DeleteFolder
    + CreateFolder
    + ListEmails
    + Clone
{
}
impl<
    T: GetAccount
        + GetFolder
        + ListAccounts
        + ListFolders
        + DeleteFolder
        + CreateFolder
        + ListEmails
        + Clone,
> Provider for T
{
}
