use pimalaya_tui::himalaya::config::Folder as HimalayaFolder;

use crate::api::folder::{Folder, FolderFlag};

mod create;
mod delete;
mod get;
mod list;

fn build_folder_params(
    folder: HimalayaFolder,
) -> (String, Option<String>, Option<Vec<FolderFlag>>, bool) {
    let all_flags = folder.desc.split(", ").collect::<Vec<&str>>();
    let (has_no_children, remaining) = all_flags
        .iter()
        .partition::<Vec<&&str>, _>(|&&flag| flag == "\\HasNoChildren");

    let has_children = has_no_children.is_empty();
    let remaining_flags = remaining
        .iter()
        .map(|&&flag| flag.to_string().into())
        .collect::<Vec<FolderFlag>>();

    (
        folder.name,
        Some(folder.desc),
        Some(remaining_flags),
        has_children,
    )
}

impl From<HimalayaFolder> for Folder {
    fn from(folder: HimalayaFolder) -> Self {
        let (name, desc, flags, has_children) = build_folder_params(folder);
        Folder::new(name, None, desc, flags, has_children)
    }
}

impl From<&HimalayaFolder> for Folder {
    fn from(folder: &HimalayaFolder) -> Self {
        let (name, desc, flags, has_children) = build_folder_params(folder.clone());
        Folder::new(name, None, desc, flags, has_children)
    }
}
