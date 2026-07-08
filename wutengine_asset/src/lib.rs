#![doc = include_str!("../README.md")]

extern crate alloc;

use alloc::sync::Arc;
use core::any::Any;
use core::any::TypeId;
use core::error::Error;
use core::fmt::Debug;
use core::fmt::Display;
use core::marker::PhantomData;
use std::path::Path;
use std::sync::LazyLock;
use std::sync::RwLock;

use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub mod assets;

#[cfg(feature = "importers")]
pub mod importers;

/// An error while to import and convert a serialized asset
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum FromSerializedAnyErr<E: Error> {
    /// Importer returned an unexpected type. Most likely an error in the importer
    #[display(
        "Importer returned invalid asset type. Should have returned {target}, but returned something else"
    )]
    Downcast {
        /// The expected target type
        target: &'static str,
    },

    /// Could not convert the deserialized asset into an actual runtime object
    #[display("Failed to load deserialized asset after importing: {_0}")]
    Conversion(E),
}

/// Trait implemented by types that can be used as a WutEngine asset
pub trait Asset: Send + Sync + Any {
    /// The serialized type of this asset
    type Serialized: SerializedAsset;

    /// The error that can be returned while loading the deserialized asset
    type FromSerializedErr: Error;

    /// Loads this asset from its serialized form
    fn from_serialized(serialized: &Self::Serialized) -> Result<Self, Self::FromSerializedErr>
    where
        Self: Sized;

    /// Loads this asset from its generic [Any] form. Should be the type of [Self::Serialized]
    fn from_serialized_any(
        serialized: &dyn Any,
    ) -> Result<Self, FromSerializedAnyErr<Self::FromSerializedErr>>
    where
        Self: Sized,
    {
        let a = serialized.downcast_ref::<Self::Serialized>().ok_or(
            FromSerializedAnyErr::Downcast {
                target: core::any::type_name::<Self::Serialized>(),
            },
        )?;

        Self::from_serialized(a).map_err(FromSerializedAnyErr::Conversion)
    }
}

/// A serialized [Asset]
pub trait SerializedAsset: Serialize + DeserializeOwned + Any {
    /// Whether to always try to serialize this asset as binary
    const PREFER_BINARY_SERIALIZATION: bool = false;
}

/// Handle to an asset
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetHandle<T> {
    /// The asset identifier
    asset_id: Option<uuid::NonNilUuid>,

    /// The loaded asset
    asset: Option<Arc<T>>,
}

impl<A> From<AssetRef<A::Serialized>> for AssetHandle<A>
where
    A: Asset,
{
    #[inline(always)]
    fn from(value: AssetRef<A::Serialized>) -> Self {
        Self::from_ref(&value)
    }
}

impl<A> From<AssetHandle<A>> for AssetRef<A::Serialized>
where
    A: Asset,
{
    #[inline(always)]
    fn from(value: AssetHandle<A>) -> Self {
        Self {
            asset_id: value.asset_id,
            _ph: PhantomData,
        }
    }
}

impl<T> Default for AssetHandle<T> {
    fn default() -> Self {
        Self {
            asset_id: None,
            asset: None,
        }
    }
}

impl<T: Asset> AssetHandle<T> {
    /// Creates a new handle from an existing asset
    pub fn new(asset: impl Into<Self>) -> Self {
        asset.into()
    }

    /// Creates a new handle from an asset reference
    pub fn from_ref(asset_ref: &AssetRef<T::Serialized>) -> Self {
        Self {
            asset_id: asset_ref.asset_id,
            asset: None,
        }
    }

    /// Creates a new handle from a serialized asset
    pub fn new_from_serialized(serialized: &T::Serialized) -> Result<Self, T::FromSerializedErr> {
        Ok(Self::new(T::from_serialized(serialized)?))
    }

    /// Returns a reference to the asset, if the asset was loaded. Otherwise returns [None]
    #[inline(always)]
    pub fn get_ref(&self) -> Option<&T> {
        self.asset.as_ref().map(Arc::as_ref)
    }

    /// Returns the cloned [Arc] containing the asset, if the asset was loaded. Otherwise returns [None]
    #[inline(always)]
    pub fn get_arc(&self) -> Option<Arc<T>> {
        self.asset.as_ref().map(Arc::clone)
    }
}

impl<T> From<T> for AssetHandle<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self {
            asset_id: None,
            asset: Some(Arc::new(value)),
        }
    }
}

impl<T> From<Option<T>> for AssetHandle<T> {
    #[inline]
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => Self::from(v),
            None => Self {
                asset_id: None,
                asset: None,
            },
        }
    }
}

/// A serializable asset reference
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AssetRef<T> {
    /// The ID of the asset
    asset_id: Option<uuid::NonNilUuid>,

    /// Phantom data for typing
    _ph: PhantomData<T>,
}

impl<T> PartialEq for AssetRef<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.asset_id == other.asset_id
    }
}

impl<T> Eq for AssetRef<T> {}

impl<T> PartialOrd for AssetRef<T> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for AssetRef<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.asset_id.cmp(&other.asset_id)
    }
}

impl<T> core::hash::Hash for AssetRef<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.asset_id.hash(state);
    }
}

/// An asset importer. Imports serialized assets from bytes, and converts them to a [SerializedAsset] type
pub trait AssetImporter: Any + Send + Sync {
    /// The type of the resulting asset
    type AssetType: Any
    where
        Self: Sized;

    /// Returns whether the given file type is supported. `file_type` will contain the extension
    /// of the type without the final dot
    fn supports_file_type(&self, file_type: &str) -> bool;

    /// Imports an asset from the given byte slice.
    /// `file_type` contains the extension without the final dot
    /// `asset_dir` can contain the parent directory that contains the asset, if any.
    ///             Can be empty if the asset was imported directly from bytes
    fn import(
        &self,
        asset_bytes: &[u8],
        file_type: &str,
        asset_dir: Option<&Path>,
    ) -> Result<Box<dyn Any>, Box<dyn Error>>;
}

/// All known importers
static IMPORTERS: LazyLock<RwLock<Vec<RegisteredImporter>>> =
    LazyLock::new(|| RwLock::new(Vec::new()));

/// The information on a registered importer
#[derive(derive_more::Debug)]
struct RegisteredImporter {
    /// The name of the importer type
    importer_name: &'static str,

    /// The [TypeId] of the imported asset produced by the importer
    asset_type: TypeId,

    /// The actual importer
    #[debug(skip)]
    importer: Box<dyn AssetImporter>,
}

/// Registers a new asset importer
pub fn register_importer<I: AssetImporter>(importer: I) {
    log::debug!(
        "Registering new importer \"{}\" for assets of type {}",
        core::any::type_name_of_val(&importer),
        core::any::type_name::<I::AssetType>()
    );

    let new_importer = RegisteredImporter {
        importer_name: core::any::type_name::<I>(),
        asset_type: TypeId::of::<I::AssetType>(),
        importer: Box::new(importer),
    };

    let mut importers = IMPORTERS.write().unwrap();

    importers.push(new_importer);
}

/// An error during asset importing
#[derive(Debug, derive_more::Error)]
pub enum ImportErr<A: Asset> {
    /// I/O error
    IO(std::io::Error),

    /// No extension could be determined, so the asset type is unknown
    UnknownExtension(#[error(not(source))] String),

    /// The list of errors returned by all importers that were tried
    ImporterErrors(#[error(not(source))] Vec<ImporterError>),

    /// An error after importing an asset, and during the loading of the imported asset into
    /// a runtime object
    LoadError(FromSerializedAnyErr<A::FromSerializedErr>),

    /// No importer supports the asset type
    UnknownAssetType(#[error(not(source))] String),
}

impl<A: Asset> Display for ImportErr<A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::IO(error) => write!(f, "I/O error while reading asset from disk: {error}"),
            Self::UnknownExtension(ext) => write!(
                f,
                "Failed to determine asset type because the asset file extension could not be determined: {ext}"
            ),
            Self::ImporterErrors(errs) => {
                writeln!(f, "Failed to import asset because all importers failed:")?;

                for err in errs {
                    writeln!(f, "\t{} failed due to: {}", err.importer, err.error)?;
                }

                Ok(())
            }
            Self::LoadError(loaderr) => write!(f, "Failed to load imported asset: {loaderr}"),
            Self::UnknownAssetType(assettype) => {
                write!(f, "No importer found for asset of type: {assettype}")
            }
        }
    }
}

/// An error returned by an importer
#[derive(Debug)]
pub struct ImporterError {
    /// The type of the importer that returned an error
    pub importer: &'static str,

    /// The returned error
    pub error: Box<dyn Error>,
}

impl<A: Asset> From<std::io::Error> for ImportErr<A> {
    fn from(value: std::io::Error) -> Self {
        ImportErr::IO(value)
    }
}

impl<A: Asset> From<FromSerializedAnyErr<A::FromSerializedErr>> for ImportErr<A> {
    fn from(value: FromSerializedAnyErr<A::FromSerializedErr>) -> Self {
        Self::LoadError(value)
    }
}

/// Imports a new asset from a given path
pub fn import<A: Asset>(asset: impl AsRef<Path>) -> Result<AssetHandle<A>, ImportErr<A>> {
    profiling::function_scope!();

    let asset_path = asset.as_ref();

    let asset_type = asset_path
        .extension()
        .and_then(|path_os| path_os.to_str())
        .ok_or_else(|| ImportErr::UnknownExtension(asset_path.to_string_lossy().to_string()))?;

    let asset_dir = asset_path.parent();

    let asset = std::fs::read(asset_path)?;

    import_from_bytes(&asset, asset_type, asset_dir)
}

/// Imports a new asset from raw bytes
pub fn import_from_bytes<A: Asset>(
    asset_bytes: &[u8],
    file_type: &str,
    asset_dir: Option<&Path>,
) -> Result<AssetHandle<A>, ImportErr<A>> {
    profiling::function_scope!();

    let registered_importers = IMPORTERS.read().unwrap();

    let mut importer_errs = Vec::new();
    let mut imported_asset = None;

    for registered_importer in registered_importers
        .iter()
        .rev()
        .filter(|imp| imp.asset_type == TypeId::of::<A::Serialized>())
        .filter(|imp| imp.importer.supports_file_type(file_type))
    {
        match registered_importer
            .importer
            .import(asset_bytes, file_type, asset_dir)
        {
            Ok(imported) => {
                log::debug!(
                    "Succesfully imported asset from importer \"{}\"",
                    registered_importer.importer_name
                );
                imported_asset = Some(imported);
                break;
            }
            Err(e) => {
                importer_errs.push(ImporterError {
                    importer: registered_importer.importer_name,
                    error: e,
                });
            }
        };
    }

    let Some(imported_asset) = imported_asset else {
        if importer_errs.is_empty() {
            return Err(ImportErr::UnknownAssetType(file_type.to_string()));
        } else {
            return Err(ImportErr::ImporterErrors(importer_errs));
        }
    };

    let loaded_asset = A::from_serialized_any(imported_asset.as_ref())?;

    Ok(AssetHandle::new(loaded_asset))
}

/// Something that can load serialized assets from disk, and provide them to the caller
/// upon request
pub trait AssetLoader {
    /// An error while loading the initial asset index with
    /// [Self::load_index]
    type LoadIndexErr: Debug + Error;

    /// An error while loading an asset with [Self::load]
    type LoadAssetErr: Debug + Error;

    /// Load the index in the given root directory. Will be called at least once before any calls
    /// to [Self::load]. Might be called again later, at which point the manager should
    /// discard its entire index and reload it from the given path
    fn load_index(&mut self, root_directory: &Path) -> Result<(), Self::LoadIndexErr>;

    /// Load the asset with the given ID. Reads should not be cached, and calls with
    /// the same
    fn load<T: Asset>(
        &mut self,
        id: uuid::Uuid,
    ) -> Result<T, LoadErr<T::FromSerializedErr, Self::LoadAssetErr>>;
}

/// An error while loading an asset with [AssetLoader::load]
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum LoadErr<A, M> {
    /// Asset was not found
    #[display("Asset {} was not found", _0)]
    NotFound(#[error(not(source))] uuid::Uuid),

    /// Loader returned an error while loading from storage
    #[display("Asset loader failed to load asset from storage: {}", _0)]
    Storage(M),

    /// Asset was loaded from storage, but could not be deserialized into a runtime
    /// asset
    #[display("Loaded asset could not be deserialized into runtime asset: {}", _0)]
    Asset(A),
}

impl<A, M> LoadErr<A, M> {
    /// Constructor for [Self::NotFound]
    #[inline(always)]
    pub const fn not_found(id: uuid::Uuid) -> Self {
        Self::NotFound(id)
    }

    /// Constructor for [Self::Manager]
    #[inline(always)]
    pub const fn manager(inner: M) -> Self {
        Self::Storage(inner)
    }

    /// Constructor for [Self::Asset]
    #[inline(always)]
    pub const fn asset(inner: A) -> Self {
        Self::Asset(inner)
    }
}
