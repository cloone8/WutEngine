//! Asset handling

use core::any::Any;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Trait implemented by types that can be used as a WutEngine asset
pub trait Asset: Send + Sync + Any {
    type Serialized: SerializedAsset<AssetType = Self>;
    type FromSerializedErr: core::error::Error;
    fn from_serialized(serialized: &Self::Serialized) -> Result<Self, Self::FromSerializedErr>
    where
        Self: Sized;
}

pub trait SerializedAsset: Serialize + DeserializeOwned + Any {
    type AssetType: Asset<Serialized = Self>;
}

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
    #[inline(always)]
    pub fn new(asset: impl Into<Self>) -> Self {
        asset.into()
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
