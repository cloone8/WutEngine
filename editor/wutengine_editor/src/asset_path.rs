//! Project asset directory relative paths

use std::path::Path;
use std::path::PathBuf;

use crate::project::asset_manager;

/// A path to a file/directory within the project asset directory (or to the directory itself)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct AssetPath(PathBuf);

impl AssetPath {
    pub(crate) fn root() -> Self {
        Self::new(asset_manager().asset_root())
    }

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

    pub(crate) fn absolute(&self) -> &Path {
        self.0.as_path()
    }

    pub(crate) fn relative(&self) -> &Path {
        let abs = self.absolute();

        abs.strip_prefix(asset_manager().asset_root())
            .expect("AssetPath should have been valid")
    }
}
