//! Assets and asset API

use alloc::sync::Arc;
use wutengine_asset_server::AutoLoad;
#[doc(inline)]
pub use wutengine_assets::*;

/// An asset given as a value, instead of as a reference to a loadable asset. Used in component parameters
#[derive(Debug, Clone)]
pub struct AssetVal<T>(AutoLoad<T>);

impl<T> AssetVal<T> {
    /// Creates a new asset value
    #[inline]
    pub fn new(val: impl Into<Self>) -> Self {
        val.into()
    }
}

impl<T> From<T> for AssetVal<T>
where
    T: Into<Arc<T>>,
{
    #[inline]
    fn from(value: T) -> Self {
        Self(AutoLoad::new_from_value(value))
    }
}

impl<T> From<Option<T>> for AssetVal<T>
where
    T: Into<AssetVal<T>>,
{
    #[inline]
    fn from(value: Option<T>) -> Self {
        match value {
            Some(val) => Self::from(val),
            None => Self(AutoLoad::new_empty()),
        }
    }
}

/// Trait for types that can be used as parameters for components that want an asset
pub trait IntoAssetParameter<T> {
    /// Converts this type into an [`AutoLoad`]
    fn into_autoload(self) -> AutoLoad<T>;
}

impl<T> IntoAssetParameter<T> for AssetVal<T> {
    #[inline]
    fn into_autoload(self) -> AutoLoad<T> {
        self.0
    }
}

impl<T> IntoAssetParameter<T> for AssetRef<T::Serialized>
where
    T: FromSerializedAsset,
{
    #[inline]
    fn into_autoload(self) -> AutoLoad<T> {
        AutoLoad::new_from_ref(&self)
    }
}
