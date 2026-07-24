//! Asset GUI

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::RwLock;

use uuid::NonNilUuid;
use wutengine::asset::AssetRef;
use wutengine::asset::SerializedAsset;
use wutengine::asset::assets::texture::SerializedTexture;
use wutengine::asset_server::AutoLoad;

use crate::assets::cache::Editor;
use crate::project;

const DEFAULT_ICON: &str = "📦";
const DEFAULT_ICON_COLOR: wutengine_egui::egui::Color32 = wutengine_egui::egui::Color32::LIGHT_BLUE;

fn default_on_open(asset_id: &uuid::NonNilUuid) {
    let Some(project_asset) = project::asset_manager().get_project_asset(asset_id) else {
        log::error!(
            "Cannot open asset {asset_id}, because it could not be found within the project"
        );
        return;
    };

    //TODO: Open in default OS program
    log::warn!(
        "Opening asset at path {}",
        project_asset.path().relative().to_string_lossy()
    );
}

static CUSTOM_GUIS: LazyLock<RwLock<HashMap<uuid::NonNilUuid, AssetGuiInfo>>> =
    LazyLock::new(|| {
        let mut map = HashMap::default();

        insert_default_custom_guis(&mut map);

        RwLock::new(map)
    });

fn insert_default_custom_guis(map: &mut HashMap<uuid::NonNilUuid, AssetGuiInfo>) {
    macro_rules! insert_gui {
        ($asset_type:ident) => {
            map.insert($asset_type::ID, AssetGuiInfo::from_trait::<$asset_type>());
        };
    }

    insert_gui!(SerializedTexture);
}

impl AssetGui for SerializedTexture {
    const ICON: &'static str = "🖼️";

    const ICON_COLOR: wutengine_egui::egui::Color32 = wutengine_egui::egui::Color32::LIGHT_GREEN;
}

/// Registers a custom asset GUI for an asset type
pub(crate) fn add_custom_asset_gui<T: AssetGui>() {
    let asset_gui = AssetGuiInfo::from_trait::<T>();

    CUSTOM_GUIS.write().unwrap().insert(T::ID, asset_gui);
}

/// The info for the custom GUI of a single asset type
#[derive(derive_more::Debug, Clone)]
pub(crate) struct AssetGuiInfo {
    /// The icon string
    pub(crate) icon: &'static str,

    /// The icon color
    pub(crate) icon_color: wutengine_egui::egui::Color32,

    /// The on-open callback
    #[debug(skip)]
    pub(crate) on_open: Arc<dyn Fn(&uuid::NonNilUuid) + Send + Sync>,
}

impl AssetGuiInfo {
    fn from_trait<T: AssetGui>() -> Self {
        Self {
            icon: T::ICON,
            icon_color: T::ICON_COLOR,
            on_open: Arc::new(|id| {
                T::on_open(&AutoLoad::new_from_ref_in(&AssetRef::from_id(*id), Editor));
            }),
        }
    }
}

impl Default for AssetGuiInfo {
    fn default() -> Self {
        type OnOpenFn = dyn Fn(&NonNilUuid) + Send + Sync;

        static DEFAULT_ON_OPEN: LazyLock<Arc<OnOpenFn>> = LazyLock::new(|| {
            Arc::new(|asset_id| {
                default_on_open(asset_id);
            })
        });

        Self {
            icon: DEFAULT_ICON,
            icon_color: DEFAULT_ICON_COLOR,
            on_open: DEFAULT_ON_OPEN.clone(),
        }
    }
}

/// Returns the GUI to use to display an asset in the editor. Returns either the custom GUI, if any,
/// or the default GUI
pub(crate) fn get_asset_gui(asset_type_id: &uuid::NonNilUuid) -> AssetGuiInfo {
    CUSTOM_GUIS
        .read()
        .unwrap()
        .get(asset_type_id)
        .cloned()
        .unwrap_or_default()
}

/// Trait to be implemented for all assets that can display a custom gui. After implementing the trait, the type must be registered using
/// [``add_custom_asset_gui``]
pub(crate) trait AssetGui: SerializedAsset {
    /// The icon string
    const ICON: &'static str = DEFAULT_ICON;

    /// The icon color
    const ICON_COLOR: wutengine_egui::egui::Color32 = DEFAULT_ICON_COLOR;

    /// "Opens" this asset. Can mean many things, depending on the type of asset. Called when, for example, the asset is double-clicked
    /// in the project library panel
    fn on_open(asset: &AutoLoad<Self, Editor>) {
        let asset_id = asset.asset_id().expect("Asset should have an ID");

        default_on_open(&asset_id);
    }
}
