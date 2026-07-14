//! Asset GUI

use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::RwLock;

use wutengine::asset::SerializedAsset;
use wutengine::asset::assets::texture::SerializedTexture;

const DEFAULT_ICON: &str = "📦";
const DEFAULT_ICON_COLOR: wutengine_egui::egui::Color32 = wutengine_egui::egui::Color32::LIGHT_BLUE;

static CUSTOM_GUIS: LazyLock<RwLock<HashMap<uuid::NonNilUuid, AssetGuiInfo>>> =
    LazyLock::new(|| {
        let mut map = HashMap::default();

        insert_default_custom_guis(&mut map);

        RwLock::new(map)
    });

fn insert_default_custom_guis(map: &mut HashMap<uuid::NonNilUuid, AssetGuiInfo>) {
    map.insert(
        SerializedTexture::ID,
        AssetGuiInfo::from_trait::<SerializedTexture>(),
    );
}

impl AssetGui for SerializedTexture {
    const ICON: &'static str = "🖼️";

    const ICON_COLOR: wutengine_egui::egui::Color32 = wutengine_egui::egui::Color32::LIGHT_GREEN;
}

pub(crate) fn add_custom_asset_gui<T: AssetGui>() {
    let asset_gui = AssetGuiInfo::from_trait::<T>();

    CUSTOM_GUIS.write().unwrap().insert(T::ID, asset_gui);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AssetGuiInfo {
    pub(crate) icon: &'static str,
    pub(crate) icon_color: wutengine_egui::egui::Color32,
}

impl AssetGuiInfo {
    fn from_trait<T: AssetGui>() -> Self {
        Self {
            icon: T::ICON,
            icon_color: T::ICON_COLOR,
        }
    }
}

impl Default for AssetGuiInfo {
    fn default() -> Self {
        Self {
            icon: DEFAULT_ICON,
            icon_color: DEFAULT_ICON_COLOR,
        }
    }
}

pub(crate) fn get_asset_gui(asset_type_id: &uuid::NonNilUuid) -> AssetGuiInfo {
    CUSTOM_GUIS
        .read()
        .unwrap()
        .get(asset_type_id)
        .copied()
        .unwrap_or_default()
}

pub(crate) trait AssetGui: SerializedAsset {
    const ICON: &'static str = DEFAULT_ICON;
    const ICON_COLOR: wutengine_egui::egui::Color32 = DEFAULT_ICON_COLOR;
}
