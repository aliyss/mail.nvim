use pimalaya_tui::himalaya::config::Folder as HimalayaFolder;

use crate::api::folder::{Folder, FolderFlag};

mod create;
mod delete;
mod expunge;
mod get;
mod list;
mod purge;

impl From<HimalayaFolder> for Folder {
    fn from(value: HimalayaFolder) -> Self {
        let has_children = !value.desc.contains("\\HasNoChildren");
        let flags = value
            .desc
            .split(", ")
            .filter_map(|flag| (flag != "\\HasNoChildren").then_some(flag.to_owned().into()))
            .collect::<Vec<FolderFlag>>();

        Self::new(
            value.name,
            None,
            Some(value.desc),
            Some(flags),
            has_children,
        )
    }
}
