use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use std::sync::Once;

use wutengine_egui::egui;
use wutengine_egui::egui::Widget;

use crate::assets::path::AssetPath;
use crate::panel::library_panel::asset_tree::AssetTreeNode;
use crate::project::asset_manager;
use crate::project::assetmanager;

use super::EditorPanel;

static SHOULD_UPDATE: AtomicBool = AtomicBool::new(false);

mod asset_tree;
mod context_menu;

/// Panel showing the asset library of this project
#[derive(Debug)]
pub(crate) struct LibraryPanel {
    file_tree: AssetTreeNode,
    context: AssetPath,
    selected: Option<AssetPath>,
}

/// General code
impl LibraryPanel {
    fn update_tree(&mut self) {
        log::debug!("Updating asset library tree");

        self.file_tree.clear();

        // First insert all the actual assets into the tree
        let asset_manager = asset_manager();
        let assets = asset_manager.asset_iter();

        for (asset_id, asset) in assets.iter() {
            let assets_gui = crate::assets::gui::get_asset_gui(&asset.asset_type());

            let asset_path = asset.path();
            let leaf = AssetTreeNode::Leaf {
                asset_id: *asset_id,
                icon: assets_gui.icon,
                icon_color: assets_gui.icon_color,
                path: asset_path.clone(),
                name: asset.name().to_string(),
            };

            self.file_tree.insert_at(&asset_path, leaf);
        }

        drop(assets);

        // Now also insert any directories, so the view matches the filesystem, and the
        // user can import assets into directories that do not yet contain any assets
        let root = asset_manager.asset_root();

        Self::insert_empty_subdirs(&mut self.file_tree, &AssetPath::new(root));
    }

    fn insert_empty_subdirs(file_tree_root: &mut AssetTreeNode, dir: &AssetPath) {
        let dir_iter = match std::fs::read_dir(dir.absolute()) {
            Ok(di) => di,
            Err(e) => {
                log::error!(
                    "Failed to read directory structure for directory {}. Empty subdirectories below this path will not be shown: {e}",
                    dir.absolute().to_string_lossy()
                );
                return;
            }
        };

        for entry in dir_iter {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    log::error!(
                        "Failed to read an entry in directory {}. Empty subdirectories below this path will not be shown: {e}",
                        dir.absolute().to_string_lossy()
                    );
                    continue;
                }
            };

            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(e) => {
                    log::error!(
                        "Failed to determine filesystem type for path {}. If the path is a directory, empty subdirectories below it will not be shown: {e}",
                        entry.path().to_string_lossy()
                    );
                    continue;
                }
            };

            if !file_type.is_dir() {
                continue;
            }

            let entry_path = AssetPath::new(entry.path());
            file_tree_root.insert_at(
                &entry_path,
                AssetTreeNode::new_empty_dir(entry_path.clone()),
            );
            Self::insert_empty_subdirs(file_tree_root, &entry_path);
        }
    }
}

/// UI Code
impl LibraryPanel {
    fn show_file_tree(&mut self, ui: &mut egui::Ui) {
        let window_response = ui
            .scope_builder(
                egui::UiBuilder::new()
                    .id_salt("FileTree")
                    .sense(egui::Sense::CLICK),
                |ui| {
                    ui.take_available_space();

                    self.file_tree
                        .show_as_tree(&mut self.context, &mut self.selected, ui);
                },
            )
            .response;

        window_response.context_menu(|ui| {
            context_menu::dir(&AssetPath::root(), ui);
        });
    }

    fn show_directory(&mut self, ui: &mut egui::Ui) {
        let root = AssetPath::root();
        let mut to_show = vec![("Assets".to_string(), root.clone())];

        let mut total_path = root;

        for component in self.context.relative().components() {
            total_path = AssetPath::new(total_path.absolute().join(component.as_os_str()));

            to_show.push((
                component.as_os_str().to_string_lossy().to_string(),
                total_path.clone(),
            ));
        }

        ui.horizontal(|ui| {
            let mut first = true;

            for (part_name, part_path) in to_show {
                if !first {
                    ui.label(">");
                }
                first = false;

                if egui::Button::new(part_name).frame(false).ui(ui).clicked() {
                    self.context = part_path;
                }
            }
        });

        ui.separator();

        let window_response = ui
            .scope_builder(
                egui::UiBuilder::new()
                    .id_salt("ShowDirectory")
                    .sense(egui::Sense::click()),
                |ui| {
                    ui.take_available_space();

                    let new_context = egui::Grid::new("DirectoryGrid")
                        .max_col_width(75.0)
                        .show(ui, |ui| {
                            self.file_tree
                                .show_as_directory(&mut self.selected, &self.context, ui)
                        })
                        .inner;

                    if let Some(new_context) = new_context {
                        self.context = new_context;
                    }
                },
            )
            .response;

        if window_response.clicked() {
            self.selected = None;
        }

        window_response.context_menu(|ui| {
            context_menu::dir(&self.context, ui);
        });
    }
}

impl EditorPanel for LibraryPanel {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Library"
    }

    fn construct(_id: super::EditorPanelId) -> Box<dyn EditorPanel>
    where
        Self: Sized,
    {
        static ADD_LISTENER: Once = Once::new();

        ADD_LISTENER.call_once(|| {
            wutengine::event::subscribe::<assetmanager::AssetCreated>(|_| {
                SHOULD_UPDATE.store(true, Ordering::Release);
            });
        });

        let asset_root = AssetPath::root();

        let mut lib_panel = Self {
            file_tree: AssetTreeNode::Branch {
                name: "Assets".to_string(),
                path: asset_root.clone(),
                children: Vec::new(),
            },
            context: asset_root,
            selected: None,
        };

        lib_panel.update_tree();

        Box::new(lib_panel)
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        if SHOULD_UPDATE.swap(false, Ordering::AcqRel) {
            self.update_tree();
        }

        egui::Panel::left("LibraryLeft").show(ui, |ui| {
            self.show_file_tree(ui);
        });

        egui::CentralPanel::default_margins().show(ui, |ui| {
            self.show_directory(ui);
        });
    }
}
