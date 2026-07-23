use wutengine_egui::egui;
use wutengine_egui::egui::Widget;

use crate::assets::path::AssetPath;
use crate::panel::library_panel;
use crate::panel::library_panel::asset_tree::AssetTreeNode;

impl AssetTreeNode {
    pub(in crate::panel::library_panel) fn show_as_directory(
        &self,
        selected: &mut Option<AssetPath>,
        context: &AssetPath,
        ui: &mut egui::Ui,
    ) -> Option<AssetPath> {
        let AssetTreeNode::Branch { path, children, .. } = self else {
            panic!("Cannot show leaf as directory");
        };

        if path != context {
            for child in children {
                match child {
                    Self::Branch { path, .. } => {
                        if context.absolute().starts_with(path.absolute()) {
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
                    response.context_menu(|ui| library_panel::context_menu::dir(path, ui));
                }
                Self::Leaf {
                    asset_id: id, path, ..
                } => {
                    response.context_menu(|ui| library_panel::context_menu::asset(id, path, ui));
                }
            }

            if response.double_clicked() {
                // Select and make context
                *selected = Some(child.path().clone());
                return Some(child.path().clone());
            }

            if response.clicked() || response.secondary_clicked() {
                // Select only
                *selected = Some(child.path().clone());
            }
        }

        None
    }
}
