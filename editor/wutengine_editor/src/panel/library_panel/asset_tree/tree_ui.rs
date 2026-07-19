use std::path::Path;

use wutengine_egui::egui;
use wutengine_egui::egui::Widget;

use crate::assets::path::AssetPath;
use crate::panel::library_panel;
use crate::panel::library_panel::asset_tree::AssetTreeNode;

impl AssetTreeNode {
    pub(in crate::panel::library_panel) fn show_as_tree(
        &self,
        context: &mut AssetPath,
        selected: &mut Option<AssetPath>,
        ui: &mut egui::Ui,
    ) {
        match self {
            Self::Branch {
                name,
                path,
                children,
            } => {
                let highlighted = context == path;
                let force_open = !highlighted && context.absolute().starts_with(path.absolute());

                let response = CollapsingTreeItem {
                    icon: self.icon(),
                    path: path.absolute(),
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
                    library_panel::context_menu::dir(path, ui);
                });
            }
            Self::Leaf {
                asset_id,
                icon,
                path,
                name,
                ..
            } => {
                let response = egui::Label::new(format!("{} {}", icon, name))
                    .selectable(false)
                    .sense(egui::Sense::click())
                    .ui(ui);

                if response.clicked() {
                    *context = path
                        .absolute()
                        .parent()
                        .map(AssetPath::new)
                        .unwrap_or_else(AssetPath::root);
                    *selected = Some(path.clone());
                }

                response.context_menu(|ui| library_panel::context_menu::asset(asset_id, path, ui));
            }
        }
    }
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
