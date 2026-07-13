#![doc = include_str!("../README.md")]

extern crate alloc;

use core::any::Any;
use core::convert::Infallible;
use core::error::Error;
use core::fmt::Debug;
use core::marker::PhantomData;

use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub mod assets;

/// A serialized asset
pub trait SerializedAsset: Serialize + DeserializeOwned + Any + Send + Sync {
    /// Hint: To obtain one, you can generate a random V4 UUID from many websites,
    /// and then use the [uuid macro](uuid::uuid) to embed it at compile time
    const ID: uuid::NonNilUuid;

    /// Whether to always try to serialize this asset as binary
    const PREFER_BINARY_SERIALIZATION: bool = false;
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

impl<T> AssetRef<T> {
    /// Returns the referenced ID
    #[inline(always)]
    pub fn get_id(&self) -> Option<uuid::NonNilUuid> {
        self.asset_id
    }
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

/// Trait for types that can be converted from a serialized asset
pub trait FromSerializedAsset: Any + Send + Sync + Sized {
    /// A conversion error
    type Error: Error + Send;

    /// The serialized form of this asset
    type Serialized: SerializedAsset;

    /// Converts a serialized asset into its runtime-compatible form
    fn from_serialized_asset(serialized: Self::Serialized) -> Result<Self, Self::Error>;
}

impl<T> FromSerializedAsset for T
where
    T: SerializedAsset,
{
    type Error = Infallible;
    type Serialized = Self;

    fn from_serialized_asset(serialized: Self::Serialized) -> Result<Self, Self::Error> {
        Ok(serialized)
    }
}
