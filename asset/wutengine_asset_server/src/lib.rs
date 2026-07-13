#![doc = include_str!("../README.md")]

extern crate alloc;

use core::any::Any;
use core::error::Error;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::RwLock;

use alloc::sync::Arc;
use wutengine_assets::AssetRef;
use wutengine_assets::FromSerializedAsset;
use wutengine_assets::SerializedAsset;
use wutengine_task::TaskHandle;
use wutengine_util::InitOnce;

static ASSET_SERVER: InitOnce<Arc<AssetServer>> = InitOnce::new_checked();

pub struct AssetServer {
    cache: RwLock<HashMap<uuid::NonNilUuid, CachedAsset>>,
    loader: Box<dyn AssetLoader>,
}

impl AssetServer {
    fn new(loader: Box<dyn AssetLoader>) -> Self {
        Self {
            loader,
            cache: RwLock::new(HashMap::default()),
        }
    }

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

    pub fn get_ref<T: FromSerializedAsset>(
        self: &Arc<Self>,
        asset_ref: &wutengine_assets::AssetRef<T::Serialized>,
    ) -> TaskHandle<Result<Arc<T>, GetAssetErr<T::Error>>> {
        let Some(asset_id) = asset_ref.get_id() else {
            return TaskHandle::from_value(Err(GetAssetErr::MissingId));
        };

        self.get_asset(&asset_id)
    }
}

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum GetAssetErr<E: Error> {
    #[display("The given asset reference has no asset ID attached")]
    MissingId,

    #[display("The asset is not of type {}", _0)]
    #[from(skip)]
    IncorrectType(#[error(not(source))] &'static str),

    #[display("The asset loader failed to load the asset from disk: {}", _0)]
    Loader(LoadAssetErr),

    #[display("Failed to deserialize the loaded asset from Postcard: {}", _0)]
    Postcard(postcard::Error),

    #[display("Failed to deserialize the loaded asset from JSON: {}", _0)]
    Json(serde_json::Error),

    #[display(
        "Failed to convert the serialized asset into the requested type after deserialization: {}",
        _0
    )]
    #[from(skip)]
    From(E),
}

/// Initializes the asset server with the given loader. If no loader is given,
/// a dummy loader is used that does not load any assets from disk
#[doc(hidden)]
pub fn init(loader: Option<Box<dyn AssetLoader>>) {
    let loader = loader.unwrap_or_else(|| Box::new(DummyLoader));

    InitOnce::init(&ASSET_SERVER, Arc::new(AssetServer::new(loader)));
}

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum LoadAssetErr {
    #[display("The asset could not be found: {}", _0)]
    NotFound(#[error(not(source))] uuid::NonNilUuid),

    #[display("The asset could not be loaded due to an I/O error: {}", _0)]
    IO(std::io::Error),

    #[display("The asset loader returned an error: {}", _0)]
    Other(Box<dyn Error + Send>),
}

pub trait AssetLoader: Send + Sync {
    fn always_binary_format(&self) -> bool {
        false
    }

    fn load_asset(&self, asset_id: &uuid::NonNilUuid) -> Result<Vec<u8>, LoadAssetErr>;
}

struct DummyLoader;

#[derive(Debug, derive_more::Display, derive_more::Error)]
#[display("No loader was enabled")]
struct UnsupportedErr;

impl AssetLoader for DummyLoader {
    fn load_asset(&self, asset_id: &uuid::NonNilUuid) -> Result<Vec<u8>, LoadAssetErr> {
        _ = asset_id;

        Err(LoadAssetErr::Other(Box::new(UnsupportedErr)))
    }
}

#[derive(Debug, Clone)]
pub struct AutoLoad<T> {
    serialized_asset_id: Option<uuid::NonNilUuid>,
    asset: OnceLock<Arc<T>>,
}

impl<T> Default for AutoLoad<T> {
    #[inline(always)]
    fn default() -> Self {
        Self::new_empty()
    }
}

impl<T> AutoLoad<T> {
    #[inline]
    pub const fn new_empty() -> Self {
        Self {
            serialized_asset_id: None,
            asset: OnceLock::new(),
        }
    }

    #[inline]
    pub fn new_from_value(val: impl Into<Arc<T>>) -> Self {
        Self {
            serialized_asset_id: None,
            asset: OnceLock::from(val.into()),
        }
    }
}

impl<T: FromSerializedAsset> AutoLoad<T> {
    #[inline]
    pub fn new_from_ref(asset_ref: AssetRef<T::Serialized>) -> Self {
        Self {
            serialized_asset_id: asset_ref.get_id(),
            asset: OnceLock::new(),
        }
    }

    #[inline(always)]
    pub const fn asset_id(&self) -> Option<uuid::NonNilUuid> {
        self.serialized_asset_id
    }

    #[inline(always)]
    pub fn get(&self) -> Arc<T> {
        self.try_get().expect("Failed to load asset")
    }

    pub fn try_get(&self) -> Option<Arc<T>> {
        if let Some(preloaded) = self.asset.get() {
            return Some(preloaded.clone());
        }

        let asset_id = self.serialized_asset_id?;

        let loaded = match ASSET_SERVER.get_asset::<T>(&asset_id).get() {
            Ok(loaded) => loaded,
            Err(e) => {
                log::error!("Failed to load asset with ID {}: {e}", asset_id);
                return None;
            }
        };

        if self.asset.set(loaded.clone()).is_err() {
            log::warn!(
                "Duplicate asset autoload initialization for asset of type {} and ID {}",
                core::any::type_name::<T>(),
                asset_id
            );
        }

        Some(loaded)
    }
}

impl<T> From<AssetRef<T::Serialized>> for AutoLoad<T>
where
    T: FromSerializedAsset,
{
    #[inline(always)]
    fn from(value: AssetRef<T::Serialized>) -> Self {
        Self::new_from_ref(value)
    }
}

impl<T> From<Arc<T>> for AutoLoad<T> {
    #[inline(always)]
    fn from(value: Arc<T>) -> Self {
        Self::new_from_value(value)
    }
}

impl<T> From<Option<T>> for AutoLoad<T>
where
    T: Into<AutoLoad<T>>,
{
    #[inline(always)]
    fn from(value: Option<T>) -> Self {
        match value {
            Some(val) => val.into(),
            None => Self::default(),
        }
    }
}

#[inline(always)]
pub fn global_asset_server() -> &'static Arc<AssetServer> {
    &ASSET_SERVER
}

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
