use wutengine_development_overlay::{DevelopmentOverlayWindow, wutengine_egui::egui};
use wutengine_graphics::wgpu;

pub(crate) fn insert_all() {
    crate::development_overlay::add_development_overlay_window(FeatureOverlay);
}

#[derive(Debug)]
struct FeatureOverlay;

impl FeatureOverlay {
    fn show_adapter(&mut self, adapter: &wgpu::AdapterInfo, ui: &mut egui::Ui) {
        ui.label(format!("Name: {}", adapter.name));
        ui.label(format!("Vendor ID: {:08x}", adapter.vendor));
        ui.label(format!("Device ID: {:08x}", adapter.device));
        ui.label(format!("Device Type: {:?}", adapter.device_type));
        ui.label(format!("PCIe Bus ID: {}", adapter.device_pci_bus_id));
        ui.label(format!("Driver Name: {}", adapter.driver));
        ui.label(format!("Driver Info: {}", adapter.driver_info));
        ui.label(format!("Backend: {}", adapter.backend.to_str()));
        ui.label(format!(
            "Subgroup min/max size: {}/{}",
            adapter.subgroup_min_size, adapter.subgroup_max_size
        ));
        ui.label(format!(
            "Transient saves memory: {}",
            adapter
                .transient_saves_memory
                .map(|b| if b { "Yes" } else { "No" })
                .unwrap_or("Unknown")
        ));
    }
    fn show_features(
        &mut self,
        features: wgpu::Features,
        device_type: wgpu::DeviceType,
        ui: &mut egui::Ui,
    ) {
        let mut all_wanted_features =
            Vec::from_iter(super::all_wanted_features(device_type).iter_names());
        all_wanted_features.sort_by_key(|v| v.0);

        let text_style_height = ui.text_style_height(&egui::TextStyle::Body);
        egui::ScrollArea::vertical()
            .max_height(text_style_height * 20.0)
            .show_rows(
                ui,
                text_style_height,
                all_wanted_features.len(),
                |ui, range| {
                    for (feature_name, feature) in all_wanted_features
                        .into_iter()
                        .skip(range.start)
                        .take(range.end - range.start)
                    {
                        let is_supported = features.contains(feature);
                        let color = if is_supported {
                            egui::ecolor::Color32::GREEN
                        } else {
                            egui::ecolor::Color32::RED
                        };

                        let icon = if is_supported { "✅" } else { "❌" };

                        ui.horizontal(|ui| {
                            ui.colored_label(color, format!("{} {}", feature_name, icon));
                        });
                    }
                },
            );
    }
    fn show_limits(&mut self, limits: &wgpu::Limits, ui: &mut egui::Ui) {
        ui.label(format!("{limits:#?}"));
    }
}

impl DevelopmentOverlayWindow for FeatureOverlay {
    fn name(&self) -> &str {
        "Graphics Features"
    }

    fn icon(&self) -> Option<&str> {
        Some("✅")
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        let config = wutengine_graphics::active_config();

        ui.label(format!("Backend: {}", config.backend));

        ui.collapsing("GPU & Driver", |ui| {
            self.show_adapter(&config.adapter, ui);
        });

        ui.collapsing("Features", |ui| {
            self.show_features(config.features, config.adapter.device_type, ui);
        });

        ui.collapsing("Limits", |ui| {
            self.show_limits(&config.limits, ui);
        });
    }
}
