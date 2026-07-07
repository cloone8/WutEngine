//! Project definition

use std::path::Path;
use std::path::PathBuf;

use wutengine_util::InitOnce;

pub(crate) mod assetmanager;

mod project_file;
pub(crate) use project_file::*;

use assetmanager::ProjectAssetManager;

pub(crate) mod create;

static PROJECT: InitOnce<Project> = InitOnce::new_checked();

/// An error while loading the project
#[derive(Debug, derive_more::Error, derive_more::From, derive_more::Display)]
pub(crate) enum LoadProjectError {
    /// Failed to load main project file
    #[display("Failed to load the main project file: {}", _0)]
    ProjectFile(ProjectFileFromDiskErr),

    /// Failed to load the asset index
    #[display("Failed to load asset index: {}", _0)]
    AssetIndex(assetmanager::LoadErr),
}

/// Loads the project from the given main project file path
pub(crate) fn load(project_file_path: &Path) -> Result<(), LoadProjectError> {
    assert!(
        !InitOnce::is_initialized(&PROJECT),
        "Project already loaded"
    );

    let _project_file = ProjectFile::from_disk(project_file_path)?;

    let root_dir = project_file_path
        .parent()
        .expect("Project file should be in a directory")
        .to_owned();

    let mut project = Project {
        name: None,
        assets: ProjectAssetManager::load(root_dir.clone())?,
        root: root_dir,
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

/// Stores the project to disk
pub(crate) fn save() -> Result<(), SaveErr> {
    PROJECT.save()
}

/// The loaded project
pub(crate) struct Project {
    name: Option<String>,
    root: PathBuf,
    assets: ProjectAssetManager,
}

impl Project {
    fn save(&self) -> Result<(), SaveErr> {
        log::info!("Saving project to disk");

        self.assets.save()?;

        Ok(())
    }
}

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub(crate) enum SaveErr {
    #[display("Failed to save asset index to disk: {}", _0)]
    AssetIndex(assetmanager::SaveErr),
}
