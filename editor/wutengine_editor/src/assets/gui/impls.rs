use wutengine::asset::assets::level::SerializedLevel;
use wutengine::asset::assets::mesh::SerializedMesh;
use wutengine::asset::assets::texture::SerializedTexture;
use wutengine::asset_server::AutoLoad;
use wutengine_egui::egui;

use crate::assets::gui::AssetGui;
use crate::assets::server::EditorProject;
use crate::project;

impl AssetGui for SerializedTexture {
    const ICON: &'static str = "🖼️";

    const ICON_COLOR: egui::Color32 = egui::Color32::LIGHT_GREEN;
}

impl AssetGui for SerializedLevel {
    const ICON: &'static str = "🗄️";

    const ICON_COLOR: egui::Color32 = egui::Color32::YELLOW;

    fn on_open(asset: &AutoLoad<Self, EditorProject>) {
        let Some(asset_id) = asset.asset_id() else {
            log::error!("Missing asset ID when opening level");
            return;
        };

        project::open_level(&asset_id);
    }
}

impl AssetGui for SerializedMesh {
    const ICON: &'static str = "🧱";

    const ICON_COLOR: wutengine_egui::egui::Color32 = egui::Color32::DARK_BLUE;
}
