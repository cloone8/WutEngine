//! Asset GUI

use wutengine::asset::SerializedAsset;

pub(crate) trait AssetGui: SerializedAsset {
    const ICON: &'static str = "📦";
    const ICON_COLOR: wutengine_egui::egui::Color32 = wutengine_egui::egui::Color32::LIGHT_BLUE;

    const NAME: &'static str;

    const SUPPORTS_MAKE_DEFAULT: bool = false;
    fn make_default() -> Self {
        unimplemented!()
    }
}
