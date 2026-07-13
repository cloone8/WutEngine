//! Asset server for coordinating and caching asset loading

use alloc::sync::Arc;
use core::any::Any;
use core::error::Error;
use std::collections::HashMap;
use std::sync::RwLock;

use wutengine_assets::FromSerializedAsset;
use wutengine_assets::SerializedAsset;
use wutengine_task::TaskHandle;

use crate::AssetLoader;
use crate::LoadAssetErr;

/// An asset server that loads and caches WutEngine assets
#[derive(derive_more::Debug)]
pub struct AssetServer {
    cache: RwLock<HashMap<uuid::NonNilUuid, CachedAsset>>,

    #[debug("loader")]
    loader: Box<dyn AssetLoader>,
}

/// Public API
impl AssetServer {
    /// Creates a new asset server that uses the given loader
    pub fn new(loader: Box<dyn AssetLoader>) -> Arc<Self> {
        Arc::new(Self {
            loader,
            cache: RwLock::new(HashMap::default()),
        })
    }

    /// Loads the given asset asynchronously. Returns a handle that yields the asset
    pub fn get_asset<T: FromSerializedAsset>(
        self: &Arc<Self>,
        asset_id: &uuid::NonNilUuid,
    ) -> TaskHandle<Result<Arc<T>, GetAssetErr<T::Error>>> {
        profiling::function_scope!(asset_id.to_string().as_str());

        let read_lock = self.cache.read().unwrap();

        if let Some(cached) = read_lock.get(asset_id) {
            return TaskHandle::from_value(
                cached
                    .try_get_as::<T>()
                    .ok_or_else(|| GetAssetErr::IncorrectType(core::any::type_name::<T>())),
            );
        }

        drop(read_lock);

        self.clone().get_uncached::<T>(*asset_id)
    }

    /// Loads the asset referenced by the given asset reference asynchronously. Returns a handle
    /// that yields the asset.
    pub fn get_ref<T: FromSerializedAsset>(
        self: &Arc<Self>,
        asset_ref: &wutengine_assets::AssetRef<T::Serialized>,
    ) -> TaskHandle<Result<Arc<T>, GetAssetErr<T::Error>>> {
        let Some(asset_id) = asset_ref.get_id() else {
            return TaskHandle::from_value(Err(GetAssetErr::MissingId));
        };

        self.get_asset(&asset_id)
    }

    /// Purge all cached assets
    pub fn purge_cache(&self) {
        *self.cache.write().unwrap() = HashMap::default(); // Make sure we also clear the memory used by the hashmap itself
    }

    /// Purge a single asset from the cache. Returns whether the asset was actually cached
    pub fn purge_asset_from_cache(&self, asset_id: &uuid::NonNilUuid) -> bool {
        self.cache.write().unwrap().remove(asset_id).is_some()
    }
}

/// Private API
impl AssetServer {
    fn get_uncached<T: FromSerializedAsset>(
        self: Arc<Self>,
        asset_id: uuid::NonNilUuid,
    ) -> TaskHandle<Result<Arc<T>, GetAssetErr<T::Error>>> {
        wutengine_task::spawn_async(async move {
            profiling::scope!("Load from loader", asset_id.to_string().as_str());

            log::info!(
                "Loading asset {} of type {} from disk",
                asset_id,
                core::any::type_name::<T>()
            );

            let asset_bytes = self.loader.load_asset(&asset_id)?;

            let asset = if T::Serialized::PREFER_BINARY_SERIALIZATION
                || self.loader.always_binary_format()
            {
                postcard::from_bytes::<T::Serialized>(&asset_bytes)?
            } else {
                serde_json::from_slice::<T::Serialized>(&asset_bytes)?
            };

            let converted_asset =
                Arc::new(T::from_serialized_asset(asset).map_err(GetAssetErr::From)?);

            let mut write_lock = self.cache.write().unwrap();

            let prev =
                write_lock.insert(asset_id, CachedAsset::from_asset(converted_asset.clone()));

            if prev.is_some() {
                log::warn!(
                    "Duplicate load for asset {}. Internal engine issue",
                    asset_id
                );
            }

            Ok(converted_asset)
        })
    }
}

/// An error while loading an asset through an [AssetServer]
#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum GetAssetErr<E: Error> {
    /// An asset reference without ID was used
    #[display("The given asset reference has no asset ID attached")]
    MissingId,

    /// Downcasting the loaded asset to a concrete type failed
    #[display("The asset is not of type {}", _0)]
    #[from(skip)]
    IncorrectType(#[error(not(source))] &'static str),

    /// The asset loader returned an error
    #[display("The asset loader failed to load the asset from disk: {}", _0)]
    Loader(LoadAssetErr),

    /// Deserializing from Postcard failed
    #[display("Failed to deserialize the loaded asset from Postcard: {}", _0)]
    Postcard(postcard::Error),

    /// Deserializing from JSON failed
    #[display("Failed to deserialize the loaded asset from JSON: {}", _0)]
    Json(serde_json::Error),

    /// Converting the serialized asset into a runtime type failed
    #[display(
        "Failed to convert the serialized asset into the requested type after deserialization: {}",
        _0
    )]
    #[from(skip)]
    From(E),
}

#[derive(Debug)]
struct CachedAsset {
    asset: Arc<dyn Any + Send + Sync>,
}

impl CachedAsset {
    fn from_asset<T: Any + Send + Sync>(asset: Arc<T>) -> Self {
        Self { asset }
    }

    fn try_get_as<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        Arc::downcast::<T>(self.asset.clone()).ok()
    }
}
