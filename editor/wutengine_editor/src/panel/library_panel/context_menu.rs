use wutengine_egui::egui;

use crate::assets;
use crate::assets::path::AssetPath;

pub(super) fn dir(root: &AssetPath, ui: &mut egui::Ui) {
    if ui.button("Import here...").clicked() {
        assets::import::import_asset_prompt(Some(root.clone()));
    };
}

pub(super) fn asset(asset_id: &uuid::NonNilUuid, path: &AssetPath, ui: &mut egui::Ui) {
    if ui.button(path.relative().to_string_lossy()).clicked() {
        log::info!("Click on {}", asset_id);
    };
}
