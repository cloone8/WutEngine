//! Container for editor panels

use we_style::PANEL_TAB_ACTIVE_COLOR;
use we_style::PANEL_TAB_BAR_COLOR;
use we_style::PANEL_TAB_INACTIVE_COLOR;
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

        egui::Frame::new()
            .fill(PANEL_TAB_BAR_COLOR)
            .inner_margin(egui::Margin {
                left: 4,
                right: 4,
                top: 4,
                bottom: 1,
            })
            .outer_margin(egui::Margin::symmetric(0, 0))
            .show(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    let mut should_move_panel = None;
                    for (i, panel) in self.panels.iter_mut().enumerate() {
                        let is_selected = self.selected == i;

                        let button_response = ui
                            .scope(|ui| {
                                {
                                    let style = ui.style_mut();
                                    style.visuals.selection.bg_fill = PANEL_TAB_ACTIVE_COLOR;
                                    style.visuals.selection.stroke.color =
                                        style.visuals.strong_text_color();
                                    style.visuals.widgets.inactive.weak_bg_fill =
                                        PANEL_TAB_INACTIVE_COLOR;
                                    style.spacing.button_padding = egui::Vec2::new(5.0, 3.0);
                                }

                                egui::Button::new(panel.name)
                                    .frame(true)
                                    .corner_radius(egui::CornerRadius {
                                        nw: 4,
                                        ne: 4,
                                        sw: 0,
                                        se: 0,
                                    })
                                    .selected(is_selected)
                                    .ui(ui)
                            })
                            .inner;

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
            });

        if self.panels.is_empty() {
            return;
        }

        let panel = self
            .panels
            .get_mut(self.selected)
            .expect("Panel index out of range. Internal engine error");

        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(6, 2))
            .show(ui, |ui| {
                ui.take_available_space();
                panel.panel.show(ui);
            });
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
