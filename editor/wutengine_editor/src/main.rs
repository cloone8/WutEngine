#![doc = include_str!("../README.md")]
#![windows_subsystem = "windows"]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use wutengine::asset::assets::level::SerializedLevel;

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

mod assets;
mod cli_args;
mod editorwindow_renderpass;
mod exit;
mod filepicker;
mod logger;
mod panel;
mod project;
mod select_project;
mod window;

/// Global egui context
static EGUI_CONTEXT: InitOnce<egui::Context> = InitOnce::new_checked();

/// Global egui resources
static EGUI_RESOURCES: InitOnce<TextureMaterialMap> = InitOnce::new_checked();

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

    load_fonts();

    egui_extras::install_image_loaders(&EGUI_CONTEXT);

    #[cfg(debug_assertions)]
    {
        //TODO: Some shit with our custom collapsible label
        EGUI_CONTEXT.all_styles_mut(|style| style.debug.warn_if_rect_changes_id = false);
    }

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
    assets::cache::init();

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

    add_default_menu_entries();

    wutengine::runtime::add_on_exit_requested_handler(exit::on_exit_requested_handler);
    wutengine::runtime::add_on_exit_handler(exit::on_exit_handler);
}

/// Adds the default menu entries
fn add_default_menu_entries() {
    we_menu::add_entry(&["File", "New Level"], 200, || {});

    we_menu::add_entry(&["File", "Exit"], u64::MAX, || {
        wutengine::runtime::exit();
    });

    we_menu::add_entry(&["Asset", "Import..."], 300, || {
        assets::import::import_asset_prompt(None);
    });

    we_menu::add_entry(&["Asset", "Level"], 400, || {
        let new_id = project::asset_manager()
            .insert_asset(
                SerializedLevel {
                    name: "Test Level".to_string(),
                    entries: vec![],
                },
                "Levels",
                "Test Level",
            )
            .unwrap();

        log::info!("New ID: {}", new_id);
    });
}

/// Loads the egui fonts
fn load_fonts() {
    let mut font_data = BTreeMap::new();

    font_data.insert(
        "FiraCode".to_string(),
        Arc::new(egui::FontData::from_static(we_fonts::FIRA_CODA_VARIABLE)),
    );

    font_data.insert(
        "DMSans".to_string(),
        Arc::new(egui::FontData::from_static(we_fonts::DMSANS_VARIABLE)),
    );

    font_data.insert(
        "DMSans Italic".to_string(),
        Arc::new(egui::FontData::from_static(
            we_fonts::DMSANS_VARIABLE_ITALIC,
        )),
    );

    font_data.insert(
        "NotoEmoji".to_string(),
        Arc::new(egui::FontData::from_static(we_fonts::NOTO_EMOJI_VARIABLE)),
    );

    font_data.insert(
        "EmojiIconFont".to_string(),
        Arc::new(egui::FontData::from_static(we_fonts::EMOJI_ICON_FONT)),
    );

    let mut families = BTreeMap::new();

    families.insert(
        egui::FontFamily::Monospace,
        vec![
            "FiraCode".to_string(),
            "DMSans".to_string(),
            "DMSans Italic".to_string(),
            "NotoEmoji".to_string(),
            "EmojiIconFont".to_string(),
        ],
    );

    families.insert(
        egui::FontFamily::Proportional,
        vec![
            "DMSans".to_string(),
            "DMSans Italic".to_string(),
            "NotoEmoji".to_string(),
            "EmojiIconFont".to_string(),
        ],
    );

    EGUI_CONTEXT.set_fonts(egui::FontDefinitions {
        font_data,
        families,
    });
}
