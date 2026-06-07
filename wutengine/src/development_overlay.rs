pub use wutengine_development_overlay::*;

use core::error::Error;
use std::collections::BTreeMap;

use serde::Deserialize;
use wutengine_egui::egui;
use wutengine_egui::egui::Color32;

#[derive(Default)]
pub(super) struct ConfigOverlay {
    set_config_key: String,
    set_config_value: String,
    set_config_error: Option<String>,
}

impl ConfigOverlay {
    fn show_value(&mut self, ui: &mut egui::Ui, key: String, value: toml::Value) {
        match value {
            toml::Value::String(s) => {
                ui.horizontal(|ui| {
                    ui.label(format!("{key}:"));
                    ui.colored_label(egui::Color32::LIGHT_YELLOW, format!("\"{s}\""));
                });
            }
            toml::Value::Integer(i) => {
                ui.horizontal(|ui| {
                    ui.label(format!("{key}:"));
                    ui.colored_label(egui::Color32::PURPLE, i.to_string());
                });
            }
            toml::Value::Float(f) => {
                ui.horizontal(|ui| {
                    ui.label(format!("{key}:"));
                    ui.colored_label(egui::Color32::LIGHT_BLUE, format!("{f}f"));
                });
            }
            toml::Value::Boolean(b) => {
                let color = if b {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::RED
                };

                ui.horizontal(|ui| {
                    ui.label(format!("{key}:"));
                    ui.colored_label(color, b.to_string());
                });
            }
            toml::Value::Datetime(datetime) => {
                ui.horizontal(|ui| {
                    ui.label(format!("{key}:"));
                    ui.colored_label(egui::Color32::ORANGE, datetime.to_string());
                });
            }
            toml::Value::Array(values) => {
                egui::CollapsingHeader::new(key)
                    .default_open(true)
                    .show(ui, |ui| {
                        for (i, value) in values.into_iter().enumerate() {
                            self.show_value(ui, i.to_string(), value);
                        }
                    });
            }
            toml::Value::Table(map) => {
                egui::CollapsingHeader::new(key)
                    .default_open(true)
                    .show(ui, |ui| {
                        for (key, value) in map {
                            self.show_value(ui, key, value);
                        }
                    });
            }
        };
    }

    fn try_set_key(key: &str, val: &str) -> Result<(), Box<dyn Error>> {
        let deser = toml::de::ValueDeserializer::parse(val)?;
        let val = toml::Value::deserialize(deser)?;
        wutengine_config::set_raw(key, val)?;

        Ok(())
    }
}

impl DevelopmentOverlayWindow for ConfigOverlay {
    fn name(&self) -> &str {
        "Config"
    }

    fn icon(&self) -> Option<&str> {
        Some("🛠️")
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        let as_ordered: BTreeMap<_, _> = wutengine_config::get_all().into_iter().collect();

        // Show existing values
        for (key, val) in as_ordered {
            self.show_value(ui, key, val);
        }

        ui.separator();

        let enter_clicked = ui.input(|i| i.key_pressed(egui::Key::Enter));

        ui.label("Set config key");

        let key_response = ui
            .horizontal(|ui| {
                let key_label_id = ui.label("Key").id;
                ui.text_edit_singleline(&mut self.set_config_key)
                    .labelled_by(key_label_id)
            })
            .inner;

        let val_response = ui
            .horizontal(|ui| {
                let value_label_id = ui.label("Value").id;
                ui.text_edit_singleline(&mut self.set_config_value)
                    .labelled_by(value_label_id)
            })
            .inner;

        if key_response.changed() || val_response.changed() {
            self.set_config_error = None;
        }

        if enter_clicked && key_response.lost_focus() {
            val_response.request_focus();
        }

        let val_entered = enter_clicked && val_response.lost_focus();

        if ui.button("Set").clicked() || val_entered {
            match Self::try_set_key(&self.set_config_key, &self.set_config_value) {
                Ok(()) => {
                    self.set_config_key.clear();
                    self.set_config_value.clear();
                }
                Err(e) => self.set_config_error = Some(e.to_string()),
            }
        }

        if let Some(err) = self.set_config_error.as_ref().map(String::as_str) {
            ui.colored_label(Color32::RED, err);
        }
    }
}
