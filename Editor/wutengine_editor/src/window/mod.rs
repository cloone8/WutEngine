//! Editor windowing

use core::num::NonZeroU32;

use wutengine::component::Component;
use wutengine::input::WindowIdentifier;
use wutengine::runtime::SystemConfig;
use wutengine::system::Phase;
use wutengine::window::Window;
use wutengine::window::WindowConfig;
use wutengine_egui::egui;

use crate::EGUI_CONTEXT;
use crate::EGUI_RESOURCES;

mod main;
pub(crate) use main::*;

pub(crate) trait EditorWindow: Send + Sync + 'static {
    fn show(&mut self, ui: &mut egui::Ui);
}

pub(crate) struct EditorWindowContainer<T> {
    editor_window: T,
}

impl<T: EditorWindow> EditorWindowContainer<T> {
    pub(crate) fn new(editor_window: T) -> Self {
        Self { editor_window }
    }

    fn run_egui(&mut self, egui_container: &EguiWindowContainer) {
        let Some(window_handle) = egui_container.window_handle else {
            return;
        };

        if !window_handle.is_ready() {
            return;
        }

        egui_container
            .egui_window
            .run_logic(&EGUI_CONTEXT, &EGUI_RESOURCES, |ui| {
                self.editor_window.show(ui);
            });
    }
}

impl<T: EditorWindow> Component for EditorWindowContainer<T> {
    fn insert_default_component_systems(manifest: &mut wutengine::runtime::SystemManifest)
    where
        Self: Sized,
    {
        let run_sys_config = SystemConfig {
            dependencies: &[],
            parallel_batch_size: Some(NonZeroU32::new(1).unwrap()),
        };

        manifest.add_system_with_config::<(&mut Self, &EguiWindowContainer)>(
            Phase::LateUpdate, // TODO: Move to Update once we can better configure inter-component system dependencies
            "Render Egui for EditorWindowContainer",
            &run_sys_config,
            |_, (this, egui_window)| {
                this.run_egui(egui_window);
            },
        );
    }
}

#[derive(Debug)]
pub(crate) struct EguiWindowContainer {
    egui_window: Box<wutengine_egui::EguiWindow>,
    window_handle: Option<Window>,
}

impl EguiWindowContainer {
    pub(crate) fn new(window_handle: Option<Window>) -> Self {
        let (input_ident, size) = match window_handle {
            Some(window_handle) => (
                WindowIdentifier::from(window_handle),
                (window_handle.get_size()),
            ),
            None => (WindowIdentifier::from(0), (1920, 1080)),
        };

        Self {
            egui_window: wutengine_egui::EguiWindow::new(
                input_ident,
                (size.0 as f32, size.1 as f32),
            ),
            window_handle,
        }
    }

    pub(crate) fn window_handle(&self) -> Option<Window> {
        self.window_handle
    }

    pub(crate) fn egui_window(&self) -> &wutengine_egui::EguiWindow {
        self.egui_window.as_ref()
    }

    fn update_parameters(&mut self) {
        let window_handle = match self.window_handle {
            Some(wh) => wh,
            None => {
                log::info!("Opening new editor window");

                let new_window = Window::create(WindowConfig {
                    title: Some(self.egui_window.title.clone()),
                    resizable: true,
                    size: (
                        (self.egui_window.surface_size_points.0 * self.egui_window.scale_factor)
                            as u32,
                        (self.egui_window.surface_size_points.1 * self.egui_window.scale_factor)
                            as u32,
                    ),
                    fullscreen: None,
                    ..Default::default()
                });

                self.window_handle = Some(new_window);
                new_window
            }
        };

        if !window_handle.is_ready() {
            return;
        }

        let egui_window_info = wutengine_egui::EguiWindowInfo {
            focused: window_handle.is_focused(),
            occluded: window_handle.is_occluded(),
            minimized: window_handle.is_minimized(),
            maximized: window_handle.is_maximized(),
        };

        let (width, height) = window_handle.get_size();
        let scale_factor = window_handle.get_scale_factor() as f32;

        self.egui_window.input_window_identifier = WindowIdentifier::from(window_handle);
        self.egui_window.window_info = egui_window_info;
        self.egui_window.surface_size_points = (
            (width as f32) / scale_factor,
            (height as f32) / scale_factor,
        );
        self.egui_window.scale_factor = scale_factor;

        self.egui_window.title = window_handle.title();
    }
}

impl Component for EguiWindowContainer {
    fn insert_default_component_systems(manifest: &mut wutengine::runtime::SystemManifest)
    where
        Self: Sized,
    {
        manifest.add_system::<&mut Self>(
            Phase::Update,
            "Update EguiWindowContainer window parameters",
            |_, this| {
                this.update_parameters();
            },
        );
    }
}
