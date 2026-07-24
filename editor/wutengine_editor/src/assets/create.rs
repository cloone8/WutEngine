//! Asset creation from the editor GUI

use std::path::Path;
use std::sync::Mutex;

use wutengine::asset::SerializedAsset;
use wutengine::asset::assets::level::SerializedLevel;
use wutengine_egui::egui;

use crate::assets::path::AssetPath;
use crate::project;

/// Trait implemented for assets that can be created straight from the editor GUI, instead
/// of having to be imported.
pub(crate) trait CreateAsset: SerializedAsset {
    /// The path to the menu entry
    const PATH: &'static [&'static str];

    /// The menu ordering location of the menu entry
    const LOCATION: u64;

    /// Returns a new default instance of the asset
    fn create_new() -> Self;
}

impl CreateAsset for SerializedLevel {
    const PATH: &'static [&'static str] = &["Level"];
    const LOCATION: u64 = 1000;

    fn create_new() -> Self {
        SerializedLevel {
            name: "New Level".to_string(),
            entries: Vec::new(),
        }
    }
}

static CREATE_ASSET_MENU: Mutex<we_menu::Menu<Path>> = Mutex::new(we_menu::Menu::new());

/// Registers a new creatable asset, which will then be shown from the create asset menus
pub(crate) fn add_creatable_asset<T: CreateAsset>() {
    let mut creatable_assets_lock = CREATE_ASSET_MENU.lock().unwrap();

    if let Err(e) = creatable_assets_lock.add_entry(T::PATH, T::LOCATION, |dir| {
        let name = T::PATH
            .last()
            .copied()
            .unwrap_or_else(|| core::any::type_name::<T>());
        let new_asset = T::create_new();

        if let Err(e) = project::asset_manager().insert_asset(&new_asset, dir, name) {
            log::error!("Could not create asset: {e}");
        }
    }) {
        log::error!("Failed to add creatable asset menu entry: {e}");
    }
}

/// Show the buttons for creating a new asset in the given directory. Should be called from within a menu of some kind.
pub(crate) fn show_buttons(dir: &AssetPath, ui: &mut egui::Ui) {
    let menu_lock = CREATE_ASSET_MENU.lock().unwrap();

    menu_lock.show(dir.absolute(), ui);
}
