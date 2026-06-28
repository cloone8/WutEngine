use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

/// A serialized project file, containing metadata about a WutEngine Editor project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ProjectFile {
    /// The editor version that was used when saving this project
    pub(crate) editor_version: String,
}

/// An error while reading a project file from disk
#[derive(Debug, derive_more::Error, derive_more::From, derive_more::Display)]
pub(crate) enum ProjectFileFromDiskErr {
    /// I/O Error
    #[display("I/O error: {}", _0)]
    IO(std::io::Error),

    /// File was corrupt
    #[display("Could not deserialize project file: {}", _0)]
    Deserialize(serde_json::Error),
}

impl ProjectFile {
    /// A new empty project file
    pub(crate) fn new() -> Self {
        Self {
            editor_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Reads a project file from disk
    pub(crate) fn from_disk(path: impl AsRef<Path>) -> Result<Self, ProjectFileFromDiskErr> {
        let path = path.as_ref();

        let project_file = std::fs::read_to_string(path)?;

        Ok(serde_json::from_str(&project_file)?)
    }
}
