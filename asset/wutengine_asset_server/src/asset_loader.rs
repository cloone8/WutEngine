//! Asset loader. Loads raw assets from disk, or any other source

use core::error::Error;

/// An error returned by an [AssetLoader]
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum LoadAssetErr {
    /// The asset could not be found
    #[display("The asset could not be found: {}", _0)]
    NotFound(#[error(not(source))] uuid::NonNilUuid),

    /// I/O error
    #[display("The asset could not be loaded due to an I/O error: {}", _0)]
    IO(std::io::Error),

    /// Any other generic error
    #[display("The asset loader returned an error: {}", _0)]
    Other(Box<dyn Error + Send>),
}

/// A type that can load assets from a source. Should return the raw asset bytes, which will then be
/// deserialized by the user
pub trait AssetLoader: Send + Sync {
    /// Whether this asset loader yields only binary serialized assets
    fn always_binary_format(&self) -> bool {
        false
    }

    /// Loads the asset with the given ID. Returns the raw, serialized bytes.
    /// Does not cache any assets, so calling this method will always perform a fresh load.
    fn load_asset(&self, asset_id: &uuid::NonNilUuid) -> Result<Vec<u8>, LoadAssetErr>;
}
