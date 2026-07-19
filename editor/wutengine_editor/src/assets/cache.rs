//! Asset caching for project assets

use alloc::sync::Arc;
use wutengine::asset_server::AssetLoader;
use wutengine::asset_server::AssetServer;

use wutengine::asset_server::LoadAssetErr;
use wutengine_util::InitOnce;

use crate::project;

static PROJECT_ASSET_SERVER: InitOnce<Arc<AssetServer>> = InitOnce::new_checked();

/// Initialize the project asset server, which uses the project file index as provided by [`crate::project::asset_manager`]
pub(crate) fn init() {
    InitOnce::init(
        &PROJECT_ASSET_SERVER,
        AssetServer::new(Box::new(ProjectAssetLoader)),
    );
}

struct ProjectAssetLoader;

impl AssetLoader for ProjectAssetLoader {
    fn load_asset(&self, asset_id: &uuid::NonNilUuid) -> Result<Vec<u8>, LoadAssetErr> {
        let asset_manager = project::asset_manager();

        let Some(project_asset) = asset_manager.get_project_asset(asset_id) else {
            return Err(LoadAssetErr::NotFound(*asset_id));
        };

        let asset_path = project_asset.path();

        std::fs::read(asset_path.absolute()).map_err(LoadAssetErr::IO)
    }
}
