use crate::utils::render::table::render::{RenderTable, RowBuilder};

pub mod commands;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderFlag {
    name: String,
}

impl From<String> for FolderFlag {
    fn from(name: String) -> Self {
        Self { name }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Folder {
    id: String,
    path: Option<Vec<String>>,
    description: Option<String>,
    flags: Option<Vec<FolderFlag>>,
    has_children: bool,
}

impl Folder {
    #[must_use]
    pub fn new(
        id: String,
        path: Option<Vec<String>>,
        description: Option<String>,
        flags: Option<Vec<FolderFlag>>,
        has_children: bool,
    ) -> Self {
        Self {
            id,
            path,
            description,
            flags,
            has_children,
        }
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn path(&self) -> &[String] {
        self.path.as_deref().unwrap_or(&[])
    }

    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    #[must_use]
    pub fn flags(&self) -> &[FolderFlag] {
        self.flags.as_deref().unwrap_or(&[])
    }

    #[must_use]
    pub fn has_children(&self) -> bool {
        self.has_children
    }
}

impl RenderTable for Vec<Folder> {
    type Item = Folder;

    fn headers(&self) -> Vec<String> {
        vec![
            "ID".to_string(),
            "Path".to_string(),
            "Description".to_string(),
            "Flags".to_string(),
            "Has Children".to_string(),
        ]
    }

    fn rows(&self) -> Vec<RowBuilder> {
        self.iter()
            .map(|folder| {
                RowBuilder::new()
                    .with_cell(folder.id.clone())
                    .with_cell(
                        folder
                            .path
                            .clone()
                            .map_or_else(String::new, |p| p.join("/")),
                    )
                    .with_cell(
                        folder
                            .description
                            .clone()
                            .unwrap_or_else(|| "None".to_string()),
                    )
                    .with_cell(
                        folder
                            .flags
                            .as_ref()
                            .map(|flags| {
                                flags
                                    .iter()
                                    .map(|f| f.name.clone())
                                    .collect::<Vec<String>>()
                                    .join(", ")
                            })
                            .unwrap_or_else(String::new),
                    )
                    .with_cell(if folder.has_children {
                        "Yes".to_string()
                    } else {
                        "No".to_string()
                    })
            })
            .collect()
    }

    fn from_headers_and_rows(headers: Vec<String>, rows: Vec<RowBuilder>) -> Self {
        let mut folders: Vec<Folder> = Vec::new();
        let id_index = headers
            .iter()
            .position(|h| h == "ID")
            .expect("Expected 'ID' header to be present in the table");
        let path_index = headers.iter().position(|h| h == "Path");
        let description_index = headers.iter().position(|h| h == "Description");
        let flags_index = headers.iter().position(|h| h == "Flags");
        let has_children_index = headers.iter().position(|h| h == "Has Children");
        for row in rows {
            let cells = row.cells;
            let id = match cells.get(id_index) {
                Some(cell) => cell.clone(),
                None => continue, // Skip rows without a name cell
            };

            let path = path_index.and_then(|index| cells.get(index).cloned());
            let description = description_index.and_then(|index| cells.get(index).cloned());
            let flags = flags_index.and_then(|index| {
                cells.get(index).map(|cell| {
                    cell.split(',')
                        .map(|flag_name| FolderFlag::from(flag_name.trim().to_string()))
                        .collect()
                })
            });
            let has_children = has_children_index
                .and_then(|index| cells.get(index).cloned())
                .is_some_and(|cell| cell.to_lowercase() == "yes");

            folders.push(Folder::new(
                id,
                path.map(|p| vec![p]),
                description,
                flags,
                has_children,
            ));
        }
        folders
    }
}
