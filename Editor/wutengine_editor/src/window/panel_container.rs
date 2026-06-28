//! Container for editor panels

use wutengine_egui::egui;
use wutengine_egui::egui::Widget;

use crate::panel::EditorPanel;
use crate::panel::EditorPanelId;

#[derive(Debug)]
pub(super) struct PanelContainer {
    selected: usize,
    panels: Vec<PanelMeta>,
}

impl PanelContainer {
    pub(super) fn new() -> Self {
        Self {
            selected: 0,
            panels: Vec::new(),
        }
    }

    pub(super) fn add<P: EditorPanel>(&mut self) -> &mut Self {
        self.panels.push(PanelMeta::new::<P>());
        self
    }

    pub(super) fn show(&mut self, ui: &mut egui::Ui) {
        self.clamp_selected();

        egui::MenuBar::new().ui(ui, |ui| {
            let mut should_move_panel = None;

            for (i, panel) in self.panels.iter_mut().enumerate() {
                let is_selected = self.selected == i;

                let button_response = egui::Button::new(panel.name).selected(is_selected).ui(ui);

                button_response.context_menu(|ui| {
                    if ui.button("Move left").clicked() {
                        should_move_panel = Some((i, i.saturating_sub(1)));
                    }

                    if ui.button("Move right").clicked() {
                        should_move_panel = Some((i, i + 1));
                    }
                });

                let select_panel = button_response.clicked();

                if select_panel {
                    self.selected = i;
                }
            }

            if let Some((from, to)) = should_move_panel {
                let to = to.min(self.panels.len() - 1);

                self.panels.swap(from, to);

                if self.selected == from {
                    self.selected = to;
                } else if self.selected == to {
                    self.selected = from;
                }
            }
        });

        ui.separator();

        if self.panels.is_empty() {
            return;
        }

        let panel = self
            .panels
            .get_mut(self.selected)
            .expect("Panel index out of range. Internal engine error");

        panel.panel.show(ui);
    }

    fn clamp_selected(&mut self) {
        self.selected = self.selected.min(self.panels.len().saturating_sub(1));
    }
}

#[derive(derive_more::Debug)]
struct PanelMeta {
    name: &'static str,

    #[debug(skip)]
    panel: Box<dyn EditorPanel>,
}

impl PanelMeta {
    fn new<P: EditorPanel>() -> Self {
        let new_id = EditorPanelId::new();

        Self {
            name: P::name(),
            panel: P::construct(new_id),
        }
    }
}
