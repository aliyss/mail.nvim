use crate::api::{
    account::commands::{GetAccount, ListAccounts},
    email::commands::{GetEmail, ListEmails},
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
    + GetEmail
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
        + GetEmail
        + Clone,
> Provider for T
{
}
