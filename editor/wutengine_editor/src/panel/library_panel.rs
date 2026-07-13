use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Once;

use wutengine_egui::egui;
use wutengine_egui::egui::Widget;

use crate::project::asset_manager;
use crate::project::assetmanager;

use super::EditorPanel;

static SHOULD_UPDATE: AtomicBool = AtomicBool::new(false);

/// Panel showing the asset library of this project
#[derive(Debug)]
pub(crate) struct LibraryPanel {
    file_tree: AssetTreeNode,
    context: PathBuf,
    selected: Option<PathBuf>,
}

/// General code
impl LibraryPanel {
    fn update_tree(&mut self) {
        log::debug!("Updating asset library tree");

        let assets = asset_manager().asset_iter();

        for (id, asset) in assets.iter() {
            let leaf = AssetTreeNode::Leaf {
                id: *id,
                path: asset.path().to_path_buf(),
                name: asset.name().to_string(),
            };

            self.file_tree.insert_at(asset.path(), leaf);
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
            context_menu_dir("", ui);
        });
    }

    fn show_directory(&mut self, ui: &mut egui::Ui) {
        let mut to_show = vec![("Assets".to_string(), PathBuf::new())];

        let mut total_path = PathBuf::new();

        for component in self.context.components() {
            total_path = total_path.join(component.as_os_str());

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
            context_menu_dir(&self.context, ui);
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

        let mut lib_panel = Self {
            file_tree: AssetTreeNode::Branch {
                name: "Assets".to_string(),
                path: PathBuf::new(),
                children: Vec::new(),
            },
            context: PathBuf::new(),
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

#[derive(Debug, Clone)]
enum AssetTreeNode {
    Branch {
        name: String,
        path: PathBuf,
        children: Vec<AssetTreeNode>,
    },
    Leaf {
        id: uuid::NonNilUuid,
        path: PathBuf,
        name: String,
    },
}

impl AssetTreeNode {
    fn name(&self) -> &str {
        match self {
            Self::Branch { name, .. } => name.as_str(),
            Self::Leaf { name, .. } => name.as_str(),
        }
    }

    fn path(&self) -> &Path {
        match self {
            Self::Branch { path, .. } => path,
            Self::Leaf { path, .. } => path,
        }
    }

    fn icon(&self) -> egui::RichText {
        match self {
            Self::Leaf { .. } => egui::RichText::new("📦").color(egui::Color32::LIGHT_BLUE),
            Self::Branch { .. } => egui::RichText::new("📁").color(egui::Color32::YELLOW),
        }
    }
    fn insert_at(&mut self, node_path: &Path, node: AssetTreeNode) {
        let AssetTreeNode::Branch { path, children, .. } = self else {
            panic!("Cannot insert at leaf node");
        };

        let should_insert_here = match node_path.parent() {
            Some(parent) => path == parent,
            None => *path == PathBuf::new(),
        };

        if should_insert_here {
            // No more children to travel down, insert the new node here
            children.push(node);
            return;
        };

        let mut to_insert = None;

        for ancestor_path in node_path.ancestors() {
            if ancestor_path == path {
                break;
            }

            to_insert = Some(ancestor_path);
        }

        let to_insert = to_insert.expect("Should have at least one subdirectory here");

        let mut new_branch = AssetTreeNode::Branch {
            name: to_insert.file_name().unwrap().to_string_lossy().to_string(),
            path: to_insert.to_path_buf(),
            children: vec![],
        };

        new_branch.insert_at(node_path, node);

        children.push(new_branch);

        children.sort_by(|a, b| a.name().cmp(b.name()));
    }

    fn show_as_tree(
        &self,
        context: &mut PathBuf,
        selected: &mut Option<PathBuf>,
        ui: &mut egui::Ui,
    ) {
        match self {
            Self::Branch {
                name,
                path,
                children,
            } => {
                let highlighted = context == path;
                let force_open = !highlighted && context.starts_with(path);

                let response = CollapsingTreeItem {
                    icon: self.icon(),
                    path,
                    name: name.as_str(),
                    highlighted,
                }
                .ui(force_open, ui, |ui| {
                    for child in children {
                        child.show_as_tree(context, selected, ui);
                    }
                });

                if response.clicked() {
                    *context = path.clone();
                }

                response.context_menu(|ui| {
                    context_menu_dir(path, ui);
                });
            }
            Self::Leaf { id, path, name } => {
                let response = egui::Label::new(format!("📦 {}", name))
                    .selectable(false)
                    .sense(egui::Sense::click())
                    .ui(ui);

                if response.clicked() {
                    *context = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
                    *selected = Some(path.clone());
                }

                response.context_menu(|ui| context_menu_asset(id, path, ui));
            }
        }
    }

    fn show_as_directory(
        &self,
        selected: &mut Option<PathBuf>,
        context: &Path,
        ui: &mut egui::Ui,
    ) -> Option<PathBuf> {
        let AssetTreeNode::Branch { path, children, .. } = self else {
            panic!("Cannot show leaf as directory");
        };

        if path != context {
            for child in children {
                match child {
                    Self::Branch { path, .. } => {
                        if context.starts_with(path) {
                            return child.show_as_directory(selected, context, ui);
                        }
                    }
                    Self::Leaf { path, .. } => {
                        if path == context {
                            // This child is the selected context, and we're the parent. Show the UI for this node
                            break;
                        }
                    }
                }
            }
        }

        // We're the selected node.

        for child in children {
            let is_selected = selected
                .as_ref()
                .is_some_and(|selected| child.path() == selected);

            let should_highlight = child.path() == context || is_selected;

            let fill = if should_highlight {
                ui.style().visuals.selection.bg_fill
            } else {
                egui::Color32::TRANSPARENT
            };

            let response = ui
                .scope_builder(egui::UiBuilder::new().sense(egui::Sense::click()), |ui| {
                    egui::Frame::new()
                        .corner_radius(5.0)
                        .fill(fill)
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                egui::Label::new(child.icon().size(30.0))
                                    .selectable(false)
                                    .ui(ui);
                                egui::Label::new(child.name()).selectable(false).ui(ui);
                            });
                        });
                })
                .response;

            match child {
                Self::Branch { path, .. } => {
                    response.context_menu(|ui| context_menu_dir(path, ui));
                }
                Self::Leaf { id, path, .. } => {
                    response.context_menu(|ui| context_menu_asset(id, path, ui));
                }
            }

            if response.double_clicked() {
                *selected = Some(child.path().to_path_buf());
                return Some(child.path().to_path_buf());
            } else {
                if response.clicked() || response.secondary_clicked() {
                    *selected = Some(child.path().to_path_buf());
                }
            }
        }

        None
    }
}

fn context_menu_dir(root: impl AsRef<Path>, ui: &mut egui::Ui) {
    let root = root.as_ref();

    if ui.button(format!("/{}", root.to_string_lossy())).clicked() {
        log::info!("Click");
    };
}

fn context_menu_asset(id: &uuid::NonNilUuid, path: impl AsRef<Path>, ui: &mut egui::Ui) {
    let path = path.as_ref();

    if ui.button(path.to_string_lossy()).clicked() {
        log::info!("Click on {}", id);
    };
}

struct CollapsingTreeItem<'a, 'b> {
    icon: egui::RichText,
    path: &'a Path,
    name: &'b str,
    highlighted: bool,
}

impl<'a, 'b> CollapsingTreeItem<'a, 'b> {
    fn ui<R>(
        self,
        force_open: bool,
        ui: &mut egui::Ui,
        body: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::Response {
        let id = ui.make_persistent_id(self.path);

        let mut state =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);

        if force_open && !state.is_open() {
            state.toggle(ui);
        }

        let (_btn, hdr, _body) = state
            .show_header(ui, |ui| {
                let fill = if self.highlighted {
                    we_style::SELECTION_BACKGROUND_COLOR
                } else {
                    egui::Color32::TRANSPARENT
                };

                ui.scope_builder(egui::UiBuilder::new().sense(egui::Sense::click()), |ui| {
                    egui::Frame::new()
                        .inner_margin(egui::Margin::same(2))
                        .corner_radius(4.0)
                        .fill(fill)
                        .show(ui, |ui| {
                            egui::Label::new(self.icon).selectable(false).ui(ui);
                            egui::Label::new(self.name).selectable(false).ui(ui);
                        });
                })
                .response
            })
            .body(|ui| {
                body(ui);
            });

        hdr.inner
    }
}
