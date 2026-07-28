//! Project asset directory relative paths

use core::fmt::Display;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use crate::project::asset_manager;

/// A path to a file/directory within the project asset directory (or to the directory itself)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct AssetPath(PathBuf);

impl AssetPath {
    #[inline(always)]
    #[expect(
        clippy::inline_always,
        reason = "Only a function because of unit tests. Otherwise would have been inlined manually"
    )]
    fn get_root_dir() -> &'static Path {
        if cfg!(test) {
            static TEST_ROOT: std::sync::LazyLock<&Path> =
                std::sync::LazyLock::new(|| Path::new("/test/assets"));

            &TEST_ROOT
        } else {
            asset_manager().asset_root()
        }
    }

    /// Returns the path to the asset folder root
    pub(crate) fn root() -> Self {
        Self::new(Self::get_root_dir())
    }

    /// Creates a new asset path from the given path.
    ///
    /// If the path is relative, it is interpreted as relative to the project asset root.
    ///
    /// If the path, after conversion to absolute form, does not reside in the project asset directory, the function panics.
    pub(crate) fn new(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let asset_root = Self::get_root_dir();

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

        let normalized = normalize_path(&abs_path);

        Self(normalized)
    }

    /// Returns the asset path as an absolute native path
    pub(crate) fn absolute(&self) -> &Path {
        self.0.as_path()
    }

    /// Returns the asset path as a native path relative to the project asset directory
    pub(crate) fn relative(&self) -> &Path {
        let abs = self.absolute();

        abs.strip_prefix(Self::get_root_dir())
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

    /// Returns whether this path is an ancestor of `other`, at any depth. Also returns `true` if `self == other`.
    pub(crate) fn is_ancestor_of(&self, other: &Self) -> bool {
        let my_path = self.absolute();
        let other_path = other.absolute();

        other_path.starts_with(my_path)
    }

    /// Returns whether `self` is the direct parent of `other`
    pub(crate) fn is_parent_of(&self, other: &Self) -> bool {
        let my_path = self.absolute();
        let other_path = other.absolute();

        other_path.parent().is_some_and(|parent| parent == my_path)
    }
}

impl Display for AssetPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_relative().fmt(f)
    }
}
/// Normalize a path, removing things like `.` and `..`. Does not resolve symlinks.
///
/// Initially taken from [cargo](https://github.com/rust-lang/cargo/blob/0158e40d8638a7de292b7242b1533caaf48cbe5f/crates/cargo-util/src/paths.rs#L86)
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();

    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().copied() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(Component::RootDir);
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if ret.ends_with(Component::ParentDir) {
                    ret.push(Component::ParentDir);
                } else {
                    let popped = ret.pop();
                    if !popped && !ret.has_root() {
                        ret.push(Component::ParentDir);
                    }
                }
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::assets::path::AssetPath;

    #[test]
    fn test_absolute() {
        let root = AssetPath::root();

        assert_eq!(Path::new("/test/assets"), root.absolute());

        assert_eq!(
            Path::new("/test/assets/my_asset.txt"),
            AssetPath::new("my_asset.txt").absolute()
        );

        assert_eq!(
            Path::new("/test/assets/subdir/my_asset.txt"),
            AssetPath::new("subdir/my_asset.txt").absolute()
        );

        assert_eq!(
            Path::new("/test/assets/subdir/my_asset.txt"),
            AssetPath::new("subdir/subdir2/../my_asset.txt").absolute()
        );
    }

    #[test]
    fn test_relative() {
        let root = AssetPath::root();

        assert_eq!(Path::new(""), root.relative());

        assert_eq!(
            Path::new("my_asset.txt"),
            AssetPath::new("my_asset.txt").relative()
        );

        assert_eq!(
            Path::new("subdir/my_asset.txt"),
            AssetPath::new("subdir/my_asset.txt").relative()
        );

        assert_eq!(
            Path::new("subdir/my_asset.txt"),
            AssetPath::new("subdir/subdir2/../my_asset.txt").relative()
        );
    }

    #[test]
    fn test_ancestor() {
        assert!(AssetPath::root().is_ancestor_of(&AssetPath::new("")));
        assert!(AssetPath::root().is_ancestor_of(&AssetPath::new("my_asset.txt")));
        assert!(AssetPath::root().is_ancestor_of(&AssetPath::new("subdir/my_asset.txt")));
        assert!(
            AssetPath::root().is_ancestor_of(&AssetPath::new("subdir/subdir2/../my_asset.txt"))
        );

        assert!(AssetPath::new("subdir").is_ancestor_of(&AssetPath::new("subdir/asset.txt")));
        assert!(
            AssetPath::new("subdir")
                .is_ancestor_of(&AssetPath::new("subdir/deep/deep/deep/deepasset.txt"))
        );

        assert!(
            !AssetPath::new("subdir/another_subdir")
                .is_ancestor_of(&AssetPath::new("subdir/asset.txt"))
        );
    }

    #[test]
    fn test_parent() {
        assert!(!AssetPath::root().is_parent_of(&AssetPath::new("")));
        assert!(AssetPath::root().is_parent_of(&AssetPath::new("my_asset.txt")));
        assert!(!AssetPath::root().is_parent_of(&AssetPath::new("subdir/my_asset.txt")));
        assert!(!AssetPath::root().is_parent_of(&AssetPath::new("subdir/subdir2/../my_asset.txt")));

        assert!(AssetPath::new("subdir").is_parent_of(&AssetPath::new("subdir/asset.txt")));
        assert!(
            !AssetPath::new("subdir")
                .is_parent_of(&AssetPath::new("subdir/deep/deep/deep/deepasset.txt"))
        );

        assert!(
            !AssetPath::new("subdir/another_subdir")
                .is_parent_of(&AssetPath::new("subdir/asset.txt"))
        );
    }
}
