//! Asset creation from the editor GUI

use std::path::Path;
use std::sync::LazyLock;
use std::sync::RwLock;

use wutengine::asset::SerializedAsset;
use wutengine::asset::assets::level::SerializedLevel;
use wutengine_egui::egui;

use crate::assets::path::AssetPath;
use crate::project;

/// Trait implemented for assets that can be created straight from the editor GUI, instead
/// of having to be imported.
pub(crate) trait CreateAsset: SerializedAsset {
    /// The human-readible name of the asset type in the menu
    const NAME: &'static str;

    /// Returns a new default instance of the asset
    fn create_new() -> Self;
}

impl CreateAsset for SerializedLevel {
    const NAME: &'static str = "Level";

    fn create_new() -> Self {
        SerializedLevel {
            name: "New Level".to_string(),
            entries: Vec::new(),
        }
    }
}

struct CreatableAsset {
    name: &'static str,
    new_fn: Box<dyn Fn(&Path) + Send + Sync>,
}

impl CreatableAsset {
    fn new<T: CreateAsset>() -> Self {
        Self {
            name: T::NAME,
            new_fn: Box::new(|dir| {
                let new_asset = T::create_new();

                if let Err(e) = project::asset_manager().insert_asset(&new_asset, dir, T::NAME) {
                    log::error!("Could not create asset: {e}");
                }
            }),
        }
    }
}

static CREATABLE_ASSETS: LazyLock<RwLock<Vec<CreatableAsset>>> = LazyLock::new(|| {
    let defaults = vec![CreatableAsset::new::<SerializedLevel>()];

    RwLock::new(defaults)
});

/// Registers a new creatable asset, which will then be shown from the create asset menus
pub(crate) fn add_creatable_asset<T: CreateAsset>() {
    let new_creatable = CreatableAsset::new::<T>();

    let mut creatable_assets_lock = CREATABLE_ASSETS.write().unwrap();

    if creatable_assets_lock
        .iter()
        .any(|creatable| creatable.name == new_creatable.name)
    {
        log::error!(
            "Not registering new creatable asset {} because a creatable asset with the same name already exists",
            new_creatable.name
        );
        return;
    }

    creatable_assets_lock.push(new_creatable);
    creatable_assets_lock.sort_by_key(|ca| ca.name);
}

/// Show the buttons for creating a new asset in the given directory. Should be called from within a menu of some kind.
pub(crate) fn create_asset_buttons(dir: &AssetPath, ui: &mut egui::Ui) {
    let creatable_assets_lock = CREATABLE_ASSETS.read().unwrap();

    for creatable_asset in &*creatable_assets_lock {
        if ui.button(creatable_asset.name).clicked() {
            (creatable_asset.new_fn)(dir.absolute());
        }
    }
}
