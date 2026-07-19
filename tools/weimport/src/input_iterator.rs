//! Input iterator

use alloc::collections::VecDeque;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

/// Iterator over asset inputs
#[derive(Debug)]
pub(crate) enum InputIterator {
    /// Empty. No more items
    Empty,

    /// Stdin only. Contains the file type
    Stdin(String),

    /// A number of input file paths
    Paths(VecDeque<PathBuf>),
}

impl InputIterator {
    /// Creates a new [`InputIterator`] that reads a file of the given type from stdin
    pub(crate) fn from_stdin(file_type: String) -> Self {
        Self::Stdin(file_type)
    }

    /// Creates a new [`InputIterator`] that reads the given input paths recursively. Input paths can be either
    /// a file path, or a directory path. Directory paths are traversed recursively, and flattened into file paths
    pub(crate) fn from_input_paths(input_paths: &[impl AsRef<Path>]) -> Self {
        let mut paths = VecDeque::new();

        for input_path in input_paths {
            Self::add_path(input_path.as_ref(), &mut paths);
        }

        Self::Paths(paths)
    }

    /// Adds a directory to this iterator, recursively
    fn add_dir_recursive(dir: &Path, out: &mut VecDeque<PathBuf>) {
        let dir_iter = match std::fs::read_dir(dir) {
            Ok(di) => di,
            Err(e) => {
                log::error!(
                    "Failed to list contents of directory {}, skipping: {e}",
                    dir.to_string_lossy()
                );
                return;
            }
        };

        for dir_entry in dir_iter {
            let dir_entry = match dir_entry {
                Ok(de) => de,
                Err(e) => {
                    log::error!(
                        "Failed to read an entry in directory {}, skipping entry: {e}",
                        dir.to_string_lossy()
                    );
                    continue;
                }
            };

            Self::add_path(&dir_entry.path(), out);
        }
    }

    /// Adds the given path to this iterator. If the path is a directory,
    /// it is traversed recursively
    fn add_path(path: &Path, out: &mut VecDeque<PathBuf>) {
        let meta = match std::fs::metadata(path) {
            Ok(meta) => meta,
            Err(e) => {
                log::error!(
                    "Failed to read metadata for path \"{}\", skipping: {e}",
                    path.to_string_lossy()
                );
                return;
            }
        };

        if meta.is_file() {
            if get_file_type_from_path(path).is_some() {
                out.push_back(path.to_path_buf());
            } else {
                log::error!(
                    "Skipping file \"{}\" because the file has no extension and thus no conclusive file type",
                    path.to_string_lossy()
                );
            }
        } else if meta.is_dir() {
            Self::add_dir_recursive(path, out);
        } else if meta.is_symlink() {
            log::warn!("Symlinks are not yet supported");
        } else {
            log::error!(
                "Unknown file type for path \"{}\", not a file, directory, or symlink according to OS",
                path.to_string_lossy()
            );
        }
    }
}

/// Returns the file type from a path, if it has an extension
fn get_file_type_from_path(path: &Path) -> Option<&str> {
    let ext = path.extension()?;

    ext.to_str()
}

/// An error produced by an element of an [`InputIterator`]
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub(crate) enum InputIteratorError {
    /// Reading input from stdin failed
    #[display("Failed to read from stdin due to error: {}", _0)]
    Stdin(std::io::Error),

    /// Reading input from a file failed
    #[display("Failed to read file at path {} due to error: {}", path.to_string_lossy(), err)]
    File {
        /// The path to the file
        path: PathBuf,

        /// The error
        err: std::io::Error,
    },
}

impl Iterator for InputIterator {
    type Item = Result<(Option<PathBuf>, String, Vec<u8>), InputIteratorError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Empty => None,
            Self::Stdin(file_type) => {
                let file_type = core::mem::take(file_type);
                *self = Self::Empty;

                let mut bytes = Vec::new();

                if let Err(e) = std::io::stdin().read_to_end(&mut bytes) {
                    return Some(Err(InputIteratorError::Stdin(e)));
                }

                Some(Ok((None, file_type, bytes)))
            }
            Self::Paths(path_bufs) => {
                let next = path_bufs.pop_front()?;

                let file_type = get_file_type_from_path(&next)
                    .expect("Invalid paths should have been filtered")
                    .to_string();

                let bytes = match std::fs::read(&next) {
                    Ok(b) => b,
                    Err(e) => {
                        return Some(Err(InputIteratorError::File { path: next, err: e }));
                    }
                };

                Some(Ok((Some(next), file_type, bytes)))
            }
        }
    }
}
