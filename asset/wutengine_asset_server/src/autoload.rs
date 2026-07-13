//! Auto-loading asset reference wrapper

use alloc::sync::Arc;
use std::sync::OnceLock;

use wutengine_assets::AssetRef;
use wutengine_assets::FromSerializedAsset;

use crate::ASSET_SERVER;
use crate::AssetServer;

/// Types that can return a reference to an [AssetServer]
pub trait AssetServerProvider: core::fmt::Debug + Clone {
    /// Returns a reference to the server
    fn server(&self) -> &Arc<AssetServer>;
}

/// The global [AssetServerProvider]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Global;

impl AssetServerProvider for Global {
    #[inline(always)]
    fn server(&self) -> &Arc<AssetServer> {
        &ASSET_SERVER
    }
}

impl AssetServerProvider for Arc<AssetServer> {
    #[inline(always)]
    fn server(&self) -> &Arc<AssetServer> {
        self
    }
}

/// Auto-loading utility for assets
#[derive(Debug, Clone)]
pub struct AutoLoad<T, S = Global> {
    serialized_asset_id: Option<uuid::NonNilUuid>,
    asset: OnceLock<Arc<T>>,
    server: S,
}

impl<T> Default for AutoLoad<T> {
    #[inline(always)]
    fn default() -> Self {
        Self::new_empty()
    }
}

impl<T> AutoLoad<T> {
    /// Creates a new empty autoloading reference for the global asset server
    #[inline]
    pub const fn new_empty() -> Self {
        Self::new_empty_in(Global)
    }

    /// Creates a new pre-loaded reference
    #[inline]
    pub fn new_from_value(val: impl Into<Arc<T>>) -> Self {
        Self::new_from_value_in(val, Global)
    }
}

impl<T: FromSerializedAsset> AutoLoad<T> {
    /// Creates a new auto-loading reference from the given asset reference, in the global
    /// asset server.
    #[inline]
    pub fn new_from_ref(asset_ref: AssetRef<T::Serialized>) -> Self {
        Self::new_from_ref_in(asset_ref, Global)
    }
}

impl<T, S: AssetServerProvider> AutoLoad<T, S> {
    /// Creates a new empty autoloading reference in the given server
    #[inline]
    pub const fn new_empty_in(server_provider: S) -> Self {
        Self {
            serialized_asset_id: None,
            asset: OnceLock::new(),
            server: server_provider,
        }
    }

    /// Creates a new pre-loaded autoloading reference from an existing value, in the given server
    #[inline]
    pub fn new_from_value_in(val: impl Into<Arc<T>>, server_provider: S) -> Self {
        Self {
            serialized_asset_id: None,
            asset: OnceLock::from(val.into()),
            server: server_provider,
        }
    }
}

impl<T: FromSerializedAsset, S: AssetServerProvider> AutoLoad<T, S> {
    /// Creates a new auto-loading reference from the given asset reference, in the given asset server
    #[inline]
    pub fn new_from_ref_in(asset_ref: AssetRef<T::Serialized>, server_provider: S) -> Self {
        Self {
            serialized_asset_id: asset_ref.get_id(),
            asset: OnceLock::new(),
            server: server_provider,
        }
    }
}

impl<T: FromSerializedAsset, S: AssetServerProvider> AutoLoad<T, S> {
    /// Returns the asset ID that this autoloader will load
    #[inline(always)]
    pub const fn asset_id(&self) -> Option<uuid::NonNilUuid> {
        self.serialized_asset_id
    }

    /// Loads and returns the referenced value. If loading fails, panics
    #[inline(always)]
    pub fn get(&self) -> Arc<T> {
        self.try_get().expect("Failed to load asset")
    }

    /// Loads and returns the referenced value, if possible
    pub fn try_get(&self) -> Option<Arc<T>> {
        if let Some(preloaded) = self.asset.get() {
            return Some(preloaded.clone());
        }

        let asset_id = self.serialized_asset_id?;

        let loaded = match self.server.server().get_asset::<T>(&asset_id).get() {
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
