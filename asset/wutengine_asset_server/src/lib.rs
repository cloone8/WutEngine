#![doc = include_str!("../README.md")]

extern crate alloc;

use alloc::sync::Arc;
use wutengine_util::InitOnce;

mod asset_server;
pub use asset_server::*;

mod autoload;
pub use autoload::*;

mod asset_loader;
pub use asset_loader::*;

/// The global (and default) asset server
static ASSET_SERVER: InitOnce<Arc<AssetServer>> = InitOnce::new_checked();

/// Initializes the global asset server with the given loader. If no loader is given,
/// a dummy loader is used that does not load any assets from disk
#[doc(hidden)]
pub fn init(loader: Option<Box<dyn AssetLoader>>) {
    let loader = loader.unwrap_or_else(|| Box::new(DummyLoader));

    InitOnce::init(&ASSET_SERVER, AssetServer::new(loader));
}

/// Returns a reference to the global asset server
#[inline(always)]
pub fn global_asset_server() -> &'static Arc<AssetServer> {
    &ASSET_SERVER
}

/// Dummy assetloader that simply fails on every load. Can be useful
/// when all assets are embedded or pre-cached
struct DummyLoader;

/// Error used by [DummyLoader]
#[derive(Debug, derive_more::Display, derive_more::Error)]
#[display("No loader was enabled")]
struct UnsupportedErr;

impl AssetLoader for DummyLoader {
    fn load_asset(&self, asset_id: &uuid::NonNilUuid) -> Result<Vec<u8>, LoadAssetErr> {
        _ = asset_id;

        Err(LoadAssetErr::Other(Box::new(UnsupportedErr)))
    }
}
