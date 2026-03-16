use crate::api::{
    account::commands::{GetAccount, ListAccounts},
    envelope::commands::ListEnvelopes,
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
    + ListEnvelopes
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
        + ListEnvelopes
        + Clone,
> Provider for T
{
}
