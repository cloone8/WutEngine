//! Project definition

use std::path::Path;
use std::path::PathBuf;

use wutengine_util::InitOnce;

mod project_file;
pub(crate) use project_file::*;

pub(crate) mod create;

static PROJECT: InitOnce<Project> = InitOnce::new();

/// An error while loading the project
#[derive(Debug, derive_more::Error, derive_more::From, derive_more::Display)]
pub(crate) enum LoadProjectError {
    /// Failed to load main project file
    #[display("Failed to load the main project file: {}", _0)]
    ProjectFile(ProjectFileFromDiskErr),
}

/// Loads the project from the given main project file path
pub(crate) fn load(project_file_path: &Path) -> Result<(), LoadProjectError> {
    assert!(
        !InitOnce::is_initialized(&PROJECT),
        "Project already loaded"
    );

    let _project_file = ProjectFile::from_disk(project_file_path)?;

    let mut project = Project {
        name: None,
        root: project_file_path
            .parent()
            .expect("Project file should be in a directory")
            .to_owned(),
    };

    project.name = project_file_path
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string());

    InitOnce::init(&PROJECT, project);

    Ok(())
}

/// Returns the name of the loaded project
pub(crate) fn name() -> Option<&'static str> {
    PROJECT.name.as_deref()
}

/// The loaded project
pub(crate) struct Project {
    name: Option<String>,
    root: PathBuf,
}
