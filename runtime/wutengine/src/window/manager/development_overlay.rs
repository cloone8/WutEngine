use wutengine_development_overlay::wutengine_egui::egui;
use wutengine_graphics::wgpu;

use super::WINDOW_MANAGER;
use crate::development_overlay::DevelopmentOverlayWindow;

#[derive(Default)]
pub(super) struct WindowManagerOverlay {}

impl DevelopmentOverlayWindow for WindowManagerOverlay {
    fn name(&self) -> &str {
        "Windows"
    }

    fn icon(&self) -> Option<&str> {
        Some("🪟")
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        let win_man = WINDOW_MANAGER.read().unwrap();
        let mut windows = Vec::new();
        for (id, info) in win_man.windows.iter() {
            windows.push(*id);

            let title = if info.is_primary {
                format!("{} [Primary Window]", info.title)
            } else {
                info.title.clone()
            };

            egui::CollapsingHeader::new(title)
                .id_salt(*id)
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(format!("ID: {id}"));
                    ui.label(format!("Size: {}x{}", info.inner_size.0, info.inner_size.1));
                    ui.label(format!("OS scale factor: {}", info.os_scale_factor));

                    ui.label(format!("Focused: {}", info.focused));
                    ui.label(format!("Occluded: {}", info.occluded));
                    ui.label(format!("Minimized: {}", info.minimized));
                    ui.label(format!("Maximized: {}", info.maximized));

                    let Some(surface_config) = info.surface.get_configuration() else {
                        return;
                    };

                    let wgpu::SurfaceConfiguration {
                        usage,
                        format,
                        width: _,
                        height: _,
                        present_mode,
                        desired_maximum_frame_latency,
                        alpha_mode,
                        view_formats,
                        color_space,
                    } = surface_config;

                    ui.label(format!("Format: {format:?}"));

                    ui.label(format!("View formats:"));
                    ui.indent(id, |ui| {
                        for tex_format in view_formats {
                            ui.label(format!("{tex_format:?}"));
                        }
                    });

                    ui.label(format!("Usages:"));
                    ui.indent(id, |ui| {
                        for (usage, _) in usage.iter_names() {
                            ui.label(usage);
                        }
                    });

                    ui.label(format!("Present mode: {present_mode:?}"));
                    ui.label(format!(
                        "Desired frame latency: {desired_maximum_frame_latency}"
                    ));

                    ui.label(format!("Alpha mode: {alpha_mode:?}"));

                    ui.label(format!("Color space: {color_space:?}"));

                    if ui.button("Reconfigure").clicked() {
                        crate::runtime::send_to_main_thread(
                            crate::runtime::MainThreadEvent::ForceSurfaceReconfigure(*id),
                        );
                    }
                });
        }

        drop(win_man);

        ui.separator();

        if ui.button("Reconfigure all").clicked() {
            for window in windows {
                crate::runtime::send_to_main_thread(
                    crate::runtime::MainThreadEvent::ForceSurfaceReconfigure(window),
                );
            }
        }
    }
}
