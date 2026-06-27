#![doc = include_str!("../../../README.md")]
#![windows_subsystem = "windows"]

extern crate alloc;

use core::num::NonZeroU32;
use std::collections::HashMap;
use std::path::PathBuf;

use clap::Parser;
use cli_args::CliArgs;
use editorwindow_renderpass::EditorWindowRenderPass;
use project::ProjectFile;
use wutengine::builtins::components::rendering::OverlayRenderPass;
use wutengine::component::Component;
use wutengine::entity::Entity;
use wutengine::graphics;
use wutengine::input::WindowIdentifier;
use wutengine::runtime;
use wutengine::runtime::FrameFrequency;
use wutengine::runtime::InitRuntimeConfig;
use wutengine::runtime::SystemConfig;
use wutengine::system::Phase;
use wutengine::time;
use wutengine::time::NANOS_PER_SECOND;
use wutengine::window::Window;
use wutengine::window::WindowConfig;
use wutengine_egui::TextureMaterialMap;
use wutengine_egui::egui;
use wutengine_util::InitOnce;

mod cli_args;
mod editorwindow_renderpass;
mod logger;
mod project;
mod select_project;

/// Global egui context
static EGUI_CONTEXT: InitOnce<egui::Context> = InitOnce::new();

/// Global egui resources
static EGUI_RESOURCES: InitOnce<TextureMaterialMap> = InitOnce::new();

/// Base update interval of the editor
const EDITOR_BASE_TICK_INTERVAL_SECS: f32 = 2.0;

#[cfg(windows)]
/// Try to attach to an already open console
fn try_attach_to_console() {
    use windows::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};

    // We can't log here because the logger is not yet initialized
    unsafe {
        _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

fn main() {
    #[cfg(windows)]
    try_attach_to_console();

    let args = CliArgs::parse();

    logger::init();

    let mut config_overrides = HashMap::default();

    config_overrides.insert("wutengine.window.triple_buffering".to_string(), true.into());

    if let Some(renderer) = args.renderer {
        let as_backend = graphics::GraphicsBackend::from(renderer);

        config_overrides.insert(
            "wutengine.graphics.backend".to_string(),
            wutengine::config::toml::Value::try_from(as_backend).unwrap(),
        );
    }

    wutengine::runtime::run(
        InitRuntimeConfig {
            frame_frequency: FrameFrequency::WaitAtMost(EDITOR_BASE_TICK_INTERVAL_SECS),
            config_overrides,
            ..Default::default()
        },
        Some(Box::new(|| post_start(args.project))),
    )
    .expect("Failure while executing WutEngine runtime");
}

/// Main startup function after the engine runtime was started
fn post_start(project: Option<PathBuf>) {
    log::info!("Starting WutEngine Editor");

    InitOnce::init(&EGUI_CONTEXT, egui::Context::default());
    InitOnce::init(&EGUI_RESOURCES, TextureMaterialMap::default());

    EGUI_CONTEXT.set_request_repaint_callback(|info| {
        _ = info;

        wutengine::runtime::request_frame();
    });

    time::set_max_frame_time((EDITOR_BASE_TICK_INTERVAL_SECS as u64 + 1) * NANOS_PER_SECOND);
    time::set_target_delta((EDITOR_BASE_TICK_INTERVAL_SECS as u64) * NANOS_PER_SECOND);

    let editor_window_renderpass_entity = Entity::spawn_transformless("Editor Window Renderpass");
    let editor_window_renderpass = OverlayRenderPass::new::<EditorWindowRenderPass>();
    editor_window_renderpass_entity.add_component(editor_window_renderpass);

    if let Some(project) = project {
        start_editor(project);
    } else {
        select_project::select_project();
    }
}

/// Starts the editor and loads the project file at the given path
fn start_editor(project: PathBuf) {
    let mut project_file = ProjectFile::from_disk(&project).expect("Failed to open project file"); //TODO: Handle properly
    project_file.project_name = project
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string());

    let initial_window_title = if let Some(proj_name) = project_file.project_name {
        format!("{proj_name} - WutEngine Editor")
    } else {
        "<unknown project> - WutEngine Editor".to_string()
    };

    let initial_window_size = (1920, 1080);

    let initial_window = Window::create(WindowConfig {
        title: Some(initial_window_title.clone()),
        resizable: true,
        size: initial_window_size,
        fullscreen: None,
        ..Default::default()
    });

    let main_editor_window_entity = Entity::spawn_transformless("Main Editor Window");
    let main_editor_window_container = EguiWindowContainer::new(Some(initial_window));

    main_editor_window_entity.add_component(main_editor_window_container);

    let main_editor_window = EditorWindowContainer::new(MainEditorWindow {});

    main_editor_window_entity.add_component(main_editor_window);
}

#[derive(Debug)]
struct EguiWindowContainer {
    egui_window: Box<wutengine_egui::EguiWindow>,
    window_handle: Option<Window>,
}

impl EguiWindowContainer {
    fn new(window_handle: Option<Window>) -> Self {
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
trait EditorWindow: Send + Sync + 'static {
    fn show(&mut self, ui: &mut egui::Ui);
}

struct EditorWindowContainer<T> {
    editor_window: T,
}

impl<T: EditorWindow> EditorWindowContainer<T> {
    fn new(editor_window: T) -> Self {
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
struct MainEditorWindow {}

impl EditorWindow for MainEditorWindow {
    fn show(&mut self, ui: &mut egui::Ui) {
        Self::show_ui(ui);
    }
}

impl MainEditorWindow {
    fn show_ui(ui: &mut egui::Ui) {
        egui::Panel::top("Top panel")
            .resizable(false)
            .show_inside(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("New Project").clicked() {
                            log::info!("New project");
                        }

                        ui.separator();

                        if ui.button("Exit").clicked() {
                            runtime::exit();
                        }
                    });
                    ui.menu_button("Edit", |ui| {
                        if ui.button("Undo").clicked() {
                            log::info!("Undo");
                        }

                        if ui.button("Redo").clicked() {
                            log::info!("Redo");
                        }
                    });
                });
            });

        egui::Panel::left("Left panel")
            .resizable(true)
            .show_inside(ui, |ui| {
                ui.take_available_space();
                ui.label("Hello from WutEngine Editor Left");
            });

        egui::Panel::right("Right panel")
            .resizable(true)
            .show_inside(ui, |ui| {
                ui.take_available_space();
                ui.label("Hello from WutEngine Editor Right");
            });

        egui::Panel::bottom("Bottom panel")
            .resizable(true)
            .default_size(250.0)
            .show_inside(ui, |ui| {
                ui.take_available_space();
                let editor_logger = logger::get_editor_logger();

                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("Level", |ui| {
                        if let Some(new_level) =
                            logger::show_log_level_picker(editor_logger.get_external_level(), ui)
                        {
                            editor_logger.set_external_level(new_level);
                            editor_logger.refilter_logs();
                        }
                    });

                    ui.menu_button("WutEngine Level", |ui| {
                        if let Some(new_level) =
                            logger::show_log_level_picker(editor_logger.get_internal_level(), ui)
                        {
                            editor_logger.set_internal_level(new_level);
                            editor_logger.refilter_logs();
                        }
                    });
                });

                ui.separator();

                editor_logger.show(ui);
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.label("Hello from WutEngine Editor");
        });
    }
}
