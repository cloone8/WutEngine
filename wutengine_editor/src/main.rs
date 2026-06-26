#![doc = include_str!("../../README.md")]

use core::num::NonZeroU32;

use wutengine::builtins::components::rendering::OverlayRenderPass;
use wutengine::component::Component;
use wutengine::entity::Entity;
use wutengine::graphics::renderpass::RenderPass;
use wutengine::graphics::wgpu;
use wutengine::hecs;
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

/// Global egui context
static EGUI_CONTEXT: InitOnce<egui::Context> = InitOnce::new();

/// Global egui resources
static EGUI_RESOURCES: InitOnce<TextureMaterialMap> = InitOnce::new();

/// Base update interval of the editor
const EDITOR_BASE_TICK_INTERVAL_SECS: f32 = 2.0;

fn main() {
    wutengine::runtime::run(
        InitRuntimeConfig {
            frame_frequency: FrameFrequency::WaitAtMost(EDITOR_BASE_TICK_INTERVAL_SECS),
            ..Default::default()
        },
        Some(Box::new(post_start)),
    )
    .expect("Failure while executing WutEngine runtime");
}

/// Main startup function after the engine runtime was started
fn post_start() {
    log::info!("Starting WutEngine Editor");

    InitOnce::init(&EGUI_CONTEXT, egui::Context::default());
    InitOnce::init(&EGUI_RESOURCES, TextureMaterialMap::default());

    EGUI_CONTEXT.set_request_repaint_callback(|info| {
        _ = info;

        wutengine::runtime::request_frame();
    });

    time::set_max_frame_time((EDITOR_BASE_TICK_INTERVAL_SECS as u64 + 1) * NANOS_PER_SECOND);
    time::set_target_delta((EDITOR_BASE_TICK_INTERVAL_SECS as u64) * NANOS_PER_SECOND);

    let initial_window_title = "WutEngine Editor".to_string();
    let initial_window_size = (1920, 1080);

    let initial_window = Window::create(WindowConfig {
        title: Some(initial_window_title.clone()),
        resizable: true,
        size: initial_window_size,
        fullscreen: None,
        ..Default::default()
    });

    let main_editor_window_entity = Entity::spawn_transformless("Main Editor Window");
    let main_editor_window = MainEditorWindow {
        title: initial_window_title,
        window_handle: initial_window,
        egui_window: wutengine_egui::EguiWindow::new(
            WindowIdentifier::from(initial_window),
            (initial_window_size.0 as f32, initial_window_size.1 as f32),
            Box::new(|ui| {
                egui::Panel::top("Top panel")
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        ui.horizontal(|ui| {
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
                    .show_inside(ui, |ui| {
                        ui.take_available_space();
                        ui.label("Bottom panel");
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.label("Hello from WutEngine Editor");
                });
            }),
        ),
    };

    main_editor_window_entity.add_component(main_editor_window);

    let editor_window_renderpass_entity = Entity::spawn_transformless("Editor Window Renderpass");
    let editor_window_renderpass = OverlayRenderPass::new::<EditorWindowRenderPass>();
    editor_window_renderpass_entity.add_component(editor_window_renderpass);
}

#[derive(Debug)]
struct MainEditorWindow {
    title: String,
    egui_window: Box<wutengine_egui::EguiWindow>,
    window_handle: Window,
}

// #[derive(Debug)]
// struct EditorWindow {
//     title: String,
//     egui_window: Box<wutengine_egui::EguiWindow>,
//     window_handle: Option<Window>,
// }

// impl Default for EditorWindow {
//     fn default() -> Self {
//         Self {
//             title: "WutEngine Editor".to_string(),
//             window_handle: None,
//             egui_window: wutengine_egui::EguiWindow::new(
//                 WindowIdentifier::new(0),
//                 (1920.0, 1080.0),
//                 Box::new(|ui| {
//                     egui::Panel::top("Top panel")
//                         .resizable(false)
//                         .show_inside(ui, |ui| {
//                             ui.label("Cute lil menu bar");
//                         });

//                     egui::Panel::left("Left panel")
//                         .resizable(true)
//                         .show_inside(ui, |ui| {
//                             ui.take_available_space();
//                             ui.label("Hello from WutEngine Editor Left");
//                         });

//                     egui::Panel::right("Right panel")
//                         .resizable(true)
//                         .show_inside(ui, |ui| {
//                             ui.take_available_space();
//                             ui.label("Hello from WutEngine Editor Right");
//                         });

//                     egui::Panel::bottom("Bottom panel")
//                         .resizable(true)
//                         .show_inside(ui, |ui| {
//                             ui.take_available_space();
//                             ui.label("Bottom panel");
//                         });

//                     egui::CentralPanel::default().show_inside(ui, |ui| {
//                         ui.label("Hello from WutEngine Editor");
//                     });
//                 }),
//             ),
//         }
//     }
// }

impl Component for MainEditorWindow {
    fn insert_default_component_systems(manifest: &mut wutengine::runtime::SystemManifest)
    where
        Self: Sized,
    {
        let update_params_system = manifest.add_system::<&mut Self>(
            Phase::Update,
            "Update EditorWindow window parameters",
            |_, this| {
                this.update_parameters();
            },
        );

        let render_sys_config = SystemConfig {
            dependencies: &[update_params_system],
            parallel_batch_size: Some(NonZeroU32::new(1).unwrap()), // Rendering egui is expensive, so make sure parallelize it as much as possible
        };

        manifest.add_system_with_config::<&mut Self>(
            Phase::Update,
            "Render Egui for EditorWindow",
            &render_sys_config,
            |_, this| {
                this.run_egui();
            },
        );
    }
}

impl MainEditorWindow {
    fn update_parameters(&mut self) {
        let egui_window_info = wutengine_egui::EguiWindowInfo {
            focused: self.window_handle.is_focused(),
            occluded: self.window_handle.is_occluded(),
            minimized: self.window_handle.is_minimized(),
            maximized: self.window_handle.is_maximized(),
        };

        let (width, height) = self.window_handle.get_size();
        let scale_factor = self.window_handle.get_scale_factor() as f32;

        self.egui_window.input_window_identifier = WindowIdentifier::from(self.window_handle);
        self.egui_window.window_info = egui_window_info;
        self.egui_window.surface_size_points = (
            (width as f32) / scale_factor,
            (height as f32) / scale_factor,
        );
        self.egui_window.scale_factor = scale_factor;

        self.egui_window.title = self.window_handle.title();
    }

    fn run_egui(&mut self) {
        self.egui_window.run_logic(&EGUI_CONTEXT, &EGUI_RESOURCES);
    }
}

#[derive(Debug)]
struct EditorWindowRenderPass {
    last_free: usize,
    to_free: Vec<egui::TextureId>,
}

impl EditorWindowRenderPass {
    const ORDER: u64 = u64::MAX / 2;
}

impl RenderPass<(Window, wgpu::Texture), hecs::World> for EditorWindowRenderPass {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Editor Window Renderpass"
    }

    fn order() -> u64
    where
        Self: Sized,
    {
        Self::ORDER
    }

    fn construct() -> Box<dyn RenderPass<(Window, wgpu::Texture), hecs::World>>
    where
        Self: Sized,
    {
        Box::new(Self {
            last_free: 0,
            to_free: Vec::new(),
        })
    }

    fn execute(
        &mut self,
        cmd: &mut wgpu::CommandEncoder,
        target: &(Window, wgpu::Texture),
        drawable: &hecs::World,
    ) {
        if self.last_free < time::frame_num() {
            self.last_free = time::frame_num();
            EGUI_RESOURCES.free_removed(self.to_free.drain(..));
        }

        let mut target_window: Option<&mut wutengine_egui::EguiWindow> = None;
        let mut query = drawable.query::<&mut MainEditorWindow>();

        for editor_window in query.iter() {
            if editor_window.window_handle == target.0 {
                target_window = Some(editor_window.egui_window.as_mut());
                break;
            }
        }

        let Some(target_window) = target_window else {
            return;
        };

        target_window.render_window(&target.1, &EGUI_RESOURCES, cmd, &mut self.to_free);
    }
}
