//! Project creation

use std::path::Path;
use std::path::PathBuf;

use super::ProjectFile;

/// An error while creating a new project
#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub(crate) enum CreateProjectErr {
    /// I/O Error
    #[display("I/O error while creating the project files: {}", _0)]
    IO(std::io::Error),

    /// Not a directory
    #[display("The given root was not a directory")]
    NotDir,

    /// Root directory does not exist
    #[display("The given root directory does not exist")]
    MissingRoot,
}

fn write_template(
    project_dir: impl AsRef<Path>,
    file_name: impl AsRef<Path>,
    content: impl AsRef<str>,
) -> Result<(), CreateProjectErr> {
    let project_dir = project_dir.as_ref();
    let file_name = file_name.as_ref();
    let content = content.as_ref();

    let file_path = project_dir.join(file_name);

    std::fs::write(file_path, content)?;

    Ok(())
}

fn write_template_files(project_dir: &Path) -> Result<(), CreateProjectErr> {
    write_template(
        project_dir,
        ".gitignore",
        include_str!("templates/gitignore_template"),
    )?;

    write_template(project_dir, "assets.json", "{}")?;

    Ok(())
}

/// Creates a new empty project within root directory `root`.
pub(crate) fn create_empty_project(name: &str, root: &Path) -> Result<PathBuf, CreateProjectErr> {
    if !std::fs::exists(root)? {
        return Err(CreateProjectErr::MissingRoot);
    }

    if !std::fs::metadata(root)?.is_dir() {
        return Err(CreateProjectErr::NotDir);
    }

    // Determine the project folder name, and create it
    let project_folder = root.join(name);
    std::fs::create_dir(&project_folder)?;

    // Create an empty assets folder
    let assets_folder = project_folder.join("assets");
    std::fs::create_dir(assets_folder)?;

    // Write the actual main project file
    let project_file = serde_json::to_string_pretty(&ProjectFile::new())
        .expect("Failed to serialize new project file");

    let project_file_path = project_folder.join(format!("{name}.we-project"));

    std::fs::write(&project_file_path, project_file)?;

    // Write any template files
    write_template_files(&project_folder)?;

    Ok(project_file_path)
}
