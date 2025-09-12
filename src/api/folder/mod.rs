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
