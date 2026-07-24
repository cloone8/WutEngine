//! Asset caching for project assets

use alloc::sync::Arc;
use wutengine::asset::AssetRef;
use wutengine::asset::FromSerializedAsset;
use wutengine::asset_server::AssetLoader;
use wutengine::asset_server::AssetServer;

use wutengine::asset_server::AssetServerProvider;
use wutengine::asset_server::GetAssetErr;
use wutengine::asset_server::LoadAssetErr;
use wutengine::task::TaskHandle;
use wutengine_util::InitOnce;

use crate::project;

static PROJECT_ASSET_SERVER: InitOnce<Arc<AssetServer>> = InitOnce::new_checked();

/// Initialize the project asset server, which uses the project file index as provided by [``crate::project::asset_manager``]
pub(crate) fn init() {
    InitOnce::init(
        &PROJECT_ASSET_SERVER,
        AssetServer::new(Box::new(ProjectAssetLoader)),
    );
}

/// Loads a project asset by its raw ID
pub(crate) fn load_id<T: FromSerializedAsset>(
    id: &uuid::NonNilUuid,
) -> TaskHandle<Result<Arc<T>, GetAssetErr<T::Error>>> {
    PROJECT_ASSET_SERVER.get_asset(id)
}

/// Loads a project asset
pub(crate) fn load_ref<T: FromSerializedAsset>(
    asset: &AssetRef<T::Serialized>,
) -> TaskHandle<Result<Arc<T>, GetAssetErr<T::Error>>> {
    PROJECT_ASSET_SERVER.get_ref::<T>(asset)
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

/// [`AssetServerProvider`] that references the editor project asset server
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct EditorProject;

impl AssetServerProvider for EditorProject {
    fn server(&self) -> &Arc<AssetServer> {
        &PROJECT_ASSET_SERVER
    }
}
