#![doc = include_str!("../README.md")]

extern crate alloc;

use alloc::sync::Arc;
use core::any::Any;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub mod assets;

#[cfg(feature = "importers")]
pub mod importers;

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum FromSerializedAnyErr<E: core::error::Error> {
    #[display("Cannot import asset of type {target} from asset of type <TODO: TYPE>")]
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

pub trait AssetImporter<T: SerializedAsset> {
    type Error: core::error::Error;

    fn supports_file_type(file_type: &str) -> bool;
    fn import(
        asset_bytes: &[u8],
        file_type: &str,
        asset_dir: Option<&Path>,
    ) -> Result<T, Self::Error>;
}
