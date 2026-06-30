#![doc = include_str!("../../../README.md")]
#![windows_subsystem = "windows"]

extern crate alloc;

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use clap::Parser;
use cli_args::CliArgs;
use editorwindow_renderpass::EditorWindowRenderPass;
use window::EditorWindowContainer;
use window::EguiWindowContainer;
use window::MainEditorWindow;
use wutengine::builtins::components::rendering::OverlayRenderPass;
use wutengine::entity::Entity;
use wutengine::graphics;
use wutengine::runtime::FrameFrequency;
use wutengine::runtime::InitRuntimeConfig;
use wutengine::time;
use wutengine::time::NANOS_PER_SECOND;
use wutengine::window::Window;
use wutengine::window::WindowConfig;
use wutengine_egui::TextureMaterialMap;
use wutengine_egui::egui;
use wutengine_util::InitOnce;

mod assetmanager;
mod cli_args;
mod editorwindow_renderpass;
mod logger;
mod menu;
mod panel;
mod project;
mod select_project;
mod window;

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
        start_editor(&project);
    } else {
        select_project::select_project();
    }
}

/// Starts the editor and loads the project file at the given path
fn start_editor(project_file_path: &Path) {
    project::load(project_file_path).expect("Failed to load project"); //TODO: Handle properly

    let initial_window_title = if let Some(proj_name) = project::name() {
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

    let main_editor_window = EditorWindowContainer::new(MainEditorWindow::new());

    main_editor_window_entity.add_component(main_editor_window);

    menu::add_default_menu_entries();
}
