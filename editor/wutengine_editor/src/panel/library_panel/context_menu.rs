use wutengine_egui::egui;

use crate::asset_path::AssetPath;
use crate::import_asset;

pub(super) fn dir(root: &AssetPath, ui: &mut egui::Ui) {
    if ui.button("Import here...").clicked() {
        import_asset::import_asset_prompt(Some(root.absolute().to_path_buf()));
    };
}

pub(super) fn asset(asset_id: &uuid::NonNilUuid, path: &AssetPath, ui: &mut egui::Ui) {
    if ui.button(path.relative().to_string_lossy()).clicked() {
        log::info!("Click on {}", asset_id);
    };
}
