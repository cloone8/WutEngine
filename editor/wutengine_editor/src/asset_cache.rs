//! Asset caching for project assets

use alloc::sync::Arc;
use core::any::Any;
use std::{collections::HashMap, sync::RwLock};

use wutengine::asset::SerializedAsset;
use wutengine_util::InitOnce;

use crate::{
    project,
    project::assetmanager::{ProjectAssetFormat, ProjectAssetId},
};

static ASSET_CACHE: InitOnce<AssetCacheManager> = InitOnce::new_checked();

/// Initialize the cache
#[doc(hidden)]
pub(crate) fn init() {
    InitOnce::init(&ASSET_CACHE, AssetCacheManager::new());
}

struct AssetCacheManager {
    cache: RwLock<HashMap<ProjectAssetId, CachedAsset>>,
}

impl AssetCacheManager {
    fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::default()),
        }
    }
}

struct CachedAsset {
    asset: Arc<dyn Any + Send + Sync>,
}

impl CachedAsset {
    fn new_from_asset<T: SerializedAsset>(asset: Arc<T>) -> Self {
        Self { asset }
    }

    fn get_as<T: SerializedAsset>(&self) -> Result<Arc<T>, LoadEditorAssetErr> {
        if let Ok(downcast) = self.asset.clone().downcast() {
            Ok(downcast)
        } else {
            Err(LoadEditorAssetErr::InvalidType(core::any::type_name::<T>()))
        }
    }
}

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub(crate) enum LoadEditorAssetErr {
    #[display("Asset was not found: {}", _0)]
    NotFound(#[error(not(source))] ProjectAssetId),

    #[display("Asset was not the requested type. Requested: {}", _0)]
    InvalidType(#[error(not(source))] &'static str),

    #[display("Asset file was missing on disk")]
    MissingOnDisk,

    #[display("I/O error: {}", _0)]
    IO(std::io::Error),

    #[display("Failed to deserialize from JSON: {}", _0)]
    Json(serde_json::Error),

    #[display("Failed to deserialize from Postcard: {}", _0)]
    Postcard(postcard::Error),
}

fn get<A: SerializedAsset>(id: ProjectAssetId) -> Result<Arc<A>, LoadEditorAssetErr> {
    let read_lock = ASSET_CACHE.cache.read().unwrap();

    if let Some(asset) = read_lock.get(&id) {
        return asset.get_as::<A>();
    };

    drop(read_lock);

    let project_asset_info = project::asset_manager()
        .get_project_asset(&id)
        .ok_or(LoadEditorAssetErr::NotFound(id))?;

    let path = project::asset_manager()
        .asset_root()
        .join(project_asset_info.path());

    if !std::fs::exists(&path)? {
        return Err(LoadEditorAssetErr::MissingOnDisk);
    }

    let asset_bytes = std::fs::read(&path)?;

    let serialized_asset = match project_asset_info.format() {
        ProjectAssetFormat::Json => serde_json::from_slice::<A>(&asset_bytes)?,
        ProjectAssetFormat::Postcard => postcard::from_bytes::<A>(&asset_bytes)?,
    };

    let as_arc = Arc::new(serialized_asset);

    let mut write_lock = ASSET_CACHE.cache.write().unwrap();

    write_lock.insert(id, CachedAsset::new_from_asset(as_arc.clone()));

    Ok(as_arc)
}
