//! Generic asset importing

use alloc::sync::Arc;
use core::any::Any;
use core::error::Error;
use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;
use wutengine_assets::SerializedAsset;
use wutengine_assets::assets::audioclip::SerializedAudioClip;
use wutengine_assets::assets::bundle::SerializedBundle;
use wutengine_assets::assets::level::SerializedLevel;
use wutengine_assets::assets::material::SerializedMaterial;
use wutengine_assets::assets::mesh::SerializedMesh;
use wutengine_assets::assets::sampler::SerializedSampler;
use wutengine_assets::assets::shader::SerializedShader;
use wutengine_assets::assets::texture::SerializedTexture;

use crate::AssetImporter;
use crate::ImageAssetImporter;
use crate::ImportedAsset;

/// Returns all default importers
pub fn default_importers() -> &'static HashMap<&'static str, Vec<Arc<Importer>>> {
    static DEFAULT_IMPORTERS: LazyLock<HashMap<&'static str, Vec<Arc<Importer>>>> =
        LazyLock::new(|| {
            let known_importers = [Importer::from_asset_importer::<ImageAssetImporter>()];

            let mut importer_map: HashMap<&str, Vec<Arc<Importer>>> = HashMap::new();

            for importer in known_importers {
                let importer = Arc::new(importer);

                for &supported_file_type in &importer.file_types {
                    importer_map
                        .entry(supported_file_type)
                        .or_default()
                        .push(importer.clone());
                }
            }

            importer_map
        });

    &DEFAULT_IMPORTERS
}

/// Returns all default asset types
pub fn default_asset_types() -> &'static HashMap<uuid::NonNilUuid, SerializedAssetType> {
    static DEFAULT_ASSET_TYPES: LazyLock<HashMap<uuid::NonNilUuid, SerializedAssetType>> =
        LazyLock::new(|| {
            let known_asset_types = [
                SerializedAssetType::new_from_asset::<SerializedTexture>(),
                SerializedAssetType::new_from_asset::<SerializedAudioClip>(),
                SerializedAssetType::new_from_asset::<SerializedBundle>(),
                SerializedAssetType::new_from_asset::<SerializedLevel>(),
                SerializedAssetType::new_from_asset::<SerializedMaterial>(),
                SerializedAssetType::new_from_asset::<SerializedMesh>(),
                SerializedAssetType::new_from_asset::<SerializedSampler>(),
                SerializedAssetType::new_from_asset::<SerializedShader>(),
            ];

            let mut asset_type_map = HashMap::new();

            for known_asset_type in known_asset_types {
                asset_type_map.insert(known_asset_type.id, known_asset_type);
            }

            asset_type_map
        });

    &DEFAULT_ASSET_TYPES
}

/// A type-erased byte asset importer function, as used in [`Importer`]
type ByteImportFn =
    dyn Fn(&str, &[u8], Option<&Path>) -> Result<Vec<ImportedAsset>, Box<dyn Error>> + Send + Sync;

/// A type-erased path asset importer function, as used in [`Importer`]
type PathImportFn = dyn Fn(&str, &Path) -> Result<Vec<ImportedAsset>, Box<dyn Error>> + Send + Sync;

/// A type-erased asset importer
#[derive(derive_more::Debug)]
pub struct Importer {
    /// The name of the importer
    name: &'static str,

    /// The supported file types
    file_types: Vec<&'static str>,

    /// The byte importer function
    #[debug(skip)]
    import_from_bytes_fn: Box<ByteImportFn>,

    /// The path importer function
    #[debug(skip)]
    import_from_path_fn: Box<PathImportFn>,
}

impl Importer {
    /// Creates a new [Importer] from an [`AssetImporter`]
    pub fn from_asset_importer<T: AssetImporter>() -> Self {
        Self {
            name: core::any::type_name::<T>(),
            file_types: T::supported_file_types(),
            import_from_bytes_fn: Box::new(|ftype, bytes, orig_path| {
                T::from_bytes(bytes, ftype, orig_path)
            }),
            import_from_path_fn: Box::new(|ftype, path| T::from_disk(ftype, path)),
        }
    }

    /// Returns the name of this importer
    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the file types supported by this importer
    #[inline]
    pub fn supported_file_types(&self) -> &[&'static str] {
        &self.file_types
    }

    /// Imports an asset from bytes, returning the type-erased list of imported assets
    #[inline]
    pub fn import_from_bytes(
        &self,
        file_type: impl AsRef<str>,
        file_bytes: impl AsRef<[u8]>,
        file_path: Option<&Path>,
    ) -> Result<Vec<ImportedAsset>, Box<dyn Error>> {
        (self.import_from_bytes_fn)(file_type.as_ref(), file_bytes.as_ref(), file_path)
    }

    /// Imports an asset from a path, returning the type-erased list of imported assets
    #[inline]
    pub fn import_from_path(
        &self,
        file_type: impl AsRef<str>,
        file_path: impl AsRef<Path>,
    ) -> Result<Vec<ImportedAsset>, Box<dyn Error>> {
        (self.import_from_path_fn)(file_type.as_ref(), file_path.as_ref())
    }
}

/// A type-erased asset serialization function, as used in [`SerializedAssetType`]
type SerializeFn =
    dyn Fn(&(dyn Any + Send + Sync)) -> Result<Vec<u8>, Box<dyn Error>> + Send + Sync;

/// A type-erased asset serialization function, as used in [`SerializedAssetType`]
type DeserializeFn =
    dyn Fn(&[u8]) -> Result<Box<dyn Any + Send + Sync>, Box<dyn Error>> + Send + Sync;

/// A type-erased [`SerializedAsset`], with pointers to its serialization functions and other config
#[derive(derive_more::Debug, Clone)]
pub struct SerializedAssetType {
    /// Asset type UUID
    id: uuid::NonNilUuid,

    /// Asset type name
    name: String,

    /// Whether this type prefers binary serialization over text serialization
    prefer_binary: bool,

    /// The binary serialization function
    #[debug(skip)]
    serialize_binary_fn: Arc<SerializeFn>,

    /// The text serialization function
    #[debug(skip)]
    serialize_text_fn: Arc<SerializeFn>,

    /// The binary deserialization function
    #[debug(skip)]
    deserialize_binary_fn: Arc<DeserializeFn>,

    /// The text deserialization function
    #[debug(skip)]
    deserialize_text_fn: Arc<DeserializeFn>,
}

impl SerializedAssetType {
    /// Create a new [SerializedAssetType] from its [`SerializedAsset`] trait
    pub fn new_from_asset<T: SerializedAsset>() -> Self {
        Self {
            id: T::ID,
            name: core::any::type_name::<T>()
                .split("::")
                .last()
                .unwrap()
                .to_lowercase()
                .to_string(),
            prefer_binary: T::PREFER_BINARY_SERIALIZATION,
            serialize_binary_fn: Arc::new(|asset| {
                let as_typed: &T = asset.downcast_ref::<T>().expect("Invalid downcast");

                Ok(postcard::to_allocvec(as_typed).map_err(Box::new)?)
            }),
            serialize_text_fn: Arc::new(|asset| {
                let as_typed: &T = asset.downcast_ref::<T>().expect("Invalid downcast");

                Ok(serde_json::to_vec_pretty(as_typed).map_err(Box::new)?)
            }),
            deserialize_binary_fn: Arc::new(|bytes| {
                let as_typed = postcard::from_bytes::<T>(bytes).map_err(Box::new)?;

                Ok(Box::new(as_typed))
            }),
            deserialize_text_fn: Arc::new(|bytes| {
                let as_typed = serde_json::from_slice::<T>(bytes).map_err(Box::new)?;

                Ok(Box::new(as_typed))
            }),
        }
    }

    /// Returns the ID of the asset type
    #[inline]
    pub fn asset_type_id(&self) -> uuid::NonNilUuid {
        self.id
    }

    /// Returns the name of the asset type
    #[inline]
    pub fn asset_type_name(&self) -> &str {
        &self.name
    }

    /// Whether this asset type prefers binary serialization
    #[inline]
    pub fn prefers_binary(&self) -> bool {
        self.prefer_binary
    }

    /// Serialize an asset of this type as binary
    #[inline]
    pub fn serialize_binary(
        &self,
        asset: &(dyn Any + Send + Sync),
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        (self.serialize_binary_fn)(asset)
    }

    /// Serialize an asset of this type as text
    #[inline]
    pub fn serialize_text(
        &self,
        asset: &(dyn Any + Send + Sync),
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        (self.serialize_text_fn)(asset)
    }

    /// Deserialize a binary-serialized asset into a generic asset type
    #[inline]
    pub fn deserialize_binary(
        &self,

        bytes: &[u8],
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn Error>> {
        (self.deserialize_binary_fn)(bytes)
    }

    /// Deserialize a text-serialized asset into a generic asset type
    #[inline]
    pub fn deserialize_text(
        &self,
        bytes: &[u8],
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn Error>> {
        (self.deserialize_text_fn)(bytes)
    }
}
