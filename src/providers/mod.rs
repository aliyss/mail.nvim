use crate::api::{
    account::commands::{GetAccount, ListAccounts},
    folder::commands::{CreateFolder, DeleteFolder, GetFolder, ListFolders},
};

pub mod himalaya;

pub trait Provider:
    GetAccount + GetFolder + ListAccounts + ListFolders + DeleteFolder + CreateFolder + Clone
{
}
impl<T: GetAccount + GetFolder + ListAccounts + ListFolders + DeleteFolder + CreateFolder + Clone>
    Provider for T
{
}
