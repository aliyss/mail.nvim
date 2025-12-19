use crate::api::{
    account::commands::{GetAccount, ListAccounts},
    folder::commands::{CreateFolder, DeleteFolder, GetFolder, ListFolders},
};

pub mod himalaya;

pub trait Provider:
    GetAccount + GetFolder + ListAccounts + ListFolders + DeleteFolder + CreateFolder
{
}
impl<T: GetAccount + GetFolder + ListAccounts + ListFolders + DeleteFolder + CreateFolder> Provider
    for T
{
}
