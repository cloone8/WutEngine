#![doc = include_str!("../README.md")]

extern crate alloc;

use alloc::sync::Arc;
use core::any::Any;
use core::any::TypeId;
use core::error::Error;
use core::fmt::Display;
use std::path::Path;
use std::sync::LazyLock;
use std::sync::RwLock;

use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub mod assets;

#[cfg(feature = "importers")]
pub mod importers;

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum FromSerializedAnyErr<E: core::error::Error> {
    #[display(
        "Importer returned invalid asset type. Should have returned {target}, but returned something else"
    )]
    Downcast { target: &'static str },

    #[display("Failed to load deserialized asset after importing: {_0}")]
    Conversion(E),
}

/// Trait implemented by types that can be used as a WutEngine asset
pub trait Asset: Send + Sync + Any {
    type Serialized: SerializedAsset;
    type FromSerializedErr: core::error::Error;
    fn from_serialized(serialized: &Self::Serialized) -> Result<Self, Self::FromSerializedErr>
    where
        Self: Sized;

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

pub trait SerializedAsset: Serialize + DeserializeOwned + Any {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetHandle<T> {
    #[serde(skip, default = "default_none")]
    asset: Option<Arc<T>>,
}

const fn default_none<T>() -> Option<Arc<T>> {
    None
}

impl<T> Default for AssetHandle<T> {
    fn default() -> Self {
        Self { asset: None }
    }
}

impl<T: Asset> AssetHandle<T> {
    pub fn new(asset: impl Into<Self>) -> Self {
        asset.into()
    }

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
            asset: Some(Arc::new(value)),
        }
    }
}

impl<T> From<Option<T>> for AssetHandle<T> {
    #[inline]
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => Self::from(v),
            None => Self { asset: None },
        }
    }
}

pub trait AssetImporter: Any + Send + Sync {
    type AssetType: Any
    where
        Self: Sized;
    fn supports_file_type(&self, file_type: &str) -> bool;
    fn import(
        &self,
        asset_bytes: &[u8],
        file_type: &str,
        asset_dir: Option<&Path>,
    ) -> Result<Box<dyn Any>, Box<dyn Error>>;
}

static IMPORTERS: LazyLock<RwLock<Vec<RegisteredImporter>>> =
    LazyLock::new(|| RwLock::new(Vec::new()));

#[derive(derive_more::Debug)]
struct RegisteredImporter {
    importer_name: &'static str,
    asset_type: TypeId,

    #[debug(skip)]
    importer: Box<dyn AssetImporter>,
}

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

#[derive(Debug, derive_more::Error)]
pub enum ImportErr<A: Asset> {
    IO(std::io::Error),
    UnknownExtension(#[error(not(source))] String),
    ImporterErrors(#[error(not(source))] Vec<ImporterError>),
    LoadError(FromSerializedAnyErr<A::FromSerializedErr>),
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

#[derive(Debug)]
pub struct ImporterError {
    pub importer: &'static str,
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

pub fn import<A: Asset>(asset: impl AsRef<Path>) -> Result<AssetHandle<A>, ImportErr<A>> {
    let asset_path = asset.as_ref();

    let asset_type = asset_path
        .extension()
        .and_then(|path_os| path_os.to_str())
        .ok_or_else(|| ImportErr::UnknownExtension(asset_path.to_string_lossy().to_string()))?;

    let asset_dir = asset_path.parent();

    let asset = std::fs::read(asset_path)?;

    import_from_bytes(&asset, asset_type, asset_dir)
}

pub fn import_from_bytes<A: Asset>(
    asset_bytes: &[u8],
    file_type: &str,
    asset_dir: Option<&Path>,
) -> Result<AssetHandle<A>, ImportErr<A>> {
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
