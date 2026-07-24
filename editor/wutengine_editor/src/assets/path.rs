//! Project asset directory relative paths

use core::fmt::Display;
use std::path::Path;
use std::path::PathBuf;

use crate::project::asset_manager;

/// A path to a file/directory within the project asset directory (or to the directory itself)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct AssetPath(PathBuf);

impl AssetPath {
    /// Returns the path to the asset folder root
    pub(crate) fn root() -> Self {
        Self::new(asset_manager().asset_root())
    }

    /// Creates a new asset path from the given path.
    ///
    /// If the path is relative, it is interpreted as relative to the project asset root.
    ///
    /// If the path, after conversion to absolute form, does not reside in the project asset directory, the function panics.
    pub(crate) fn new(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let asset_root = asset_manager().asset_root();

        let abs_path = if path.is_absolute() {
            assert!(
                path.starts_with(asset_root),
                "Path does not lie within the project asset root: {}",
                path.to_string_lossy()
            );

            path.to_path_buf()
        } else {
            let absolute = asset_root.join(path);

            // Check if the path still lies within the root, because joining may have placed it outside the root if the
            // path contained many ".." components
            assert!(
                absolute.starts_with(asset_root),
                "Path does not lie within the project asset root: {}",
                absolute.to_string_lossy()
            );

            absolute
        };

        Self(abs_path)
    }

    /// Returns the asset path as an absolute native path
    pub(crate) fn absolute(&self) -> &Path {
        self.0.as_path()
    }

    /// Returns the asset path as a native path relative to the project asset directory
    pub(crate) fn relative(&self) -> &Path {
        let abs = self.absolute();

        abs.strip_prefix(asset_manager().asset_root())
            .expect("AssetPath should have been valid")
    }

    /// Returns a formatter that displays this asset path as an absolute path
    pub(crate) fn fmt_absolute(&self) -> impl core::fmt::Debug + core::fmt::Display {
        core::fmt::from_fn(|fmt| self.absolute().to_string_lossy().fmt(fmt))
    }

    /// Returns a formatter that displays this asset path as a relative path
    pub(crate) fn fmt_relative(&self) -> impl core::fmt::Debug + core::fmt::Display {
        core::fmt::from_fn(|fmt| self.relative().to_string_lossy().fmt(fmt))
    }
}

impl Display for AssetPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_relative().fmt(f)
    }
}
