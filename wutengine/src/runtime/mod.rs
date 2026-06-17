//! The main WutEngine runtime, responsible for the application lifecycle

use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::Instant;
use wutengine_util::assert_main_thread;

use derive_more::{Display, Error, From};
use winit::error::EventLoopError;
use wutengine_graphics::wgpu;

use crate::audio;
use crate::builtins::components::rendering::Camera;
use crate::builtins::components::rendering::CameraRenderPass;
use crate::builtins::components::rendering::GlobalRenderPass;
use crate::entity::{self, EntityManager};
use crate::graphics::DrawCommand;
use crate::graphics::RenderPassInfo;
use crate::input;
use crate::physics2d;
use crate::physics3d;
use crate::system::{self, Phase, SystemManager};
use crate::window::{self, Window};
use crate::{graphics, time, world};
use wutengine_util::{self, InitOnce};

use rayon::prelude::*;

mod system_builder;
mod winit_app;

pub use system_builder::*;

pub(crate) use winit_app::WinitEvent;

static EVENT_LOOP_PROXY: InitOnce<winit::event_loop::EventLoopProxy<WinitEvent>> = InitOnce::new();
static WUTENGINE_RUNNING: AtomicBool = AtomicBool::new(false);

/// Notifies the main [winit] event loop of a given event.
///
/// If the loop was already closed, does nothing and logs an error
pub(crate) fn notify_event_loop(event: WinitEvent) {
    if let Err(e) = EVENT_LOOP_PROXY.send_event(event) {
        log::error!(
            "Failed to notify event loop of event {:#?} because it was already closed",
            e.0
        );
    }
}

/// Data only relevant before/during application initialization in [winit::application::ApplicationHandler::resumed]
pub(crate) struct InitializationData {
    post_start_callback: Option<Box<dyn FnOnce()>>,
}

/// The main WutEngine runtime
pub(crate) struct Runtime {
    /// Used for frame pacing and FPS limiting
    frame_pacer: window::pacer::FramePacer,

    /// Set to `true` if the `resumed` event was sent by [winit]
    initialization_data: Option<Box<InitializationData>>,

    /// The entity manager. Spawns entities and components
    entity_manager: EntityManager,

    /// Contains the systems and their schedule
    systems: SystemManager,

    /// The receiving end of the graphics command queue
    draw_commands: Receiver<crate::graphics::DrawCommand>,
}

/// An error while starting the WutEngine runtime with [run]
#[derive(Debug, Error, Display, From)]
pub enum RuntimeStartErr {
    /// Another WutEngine runtime was already started in this process
    #[display("Another WutEngine runtime was already started in this process")]
    AlreadyRunning,

    /// Error running the winit event loop
    #[display("Error running the winit event loop: {_0}")]
    EventLoop(EventLoopError),
}

/// The configuration used to start the WutEngine runtime
#[derive(Debug, Clone)]
pub struct InitRuntimeConfig {
    /// The path to a config file, used for population the initial values of the [crate::config] module
    pub config_file: Option<PathBuf>,

    /// Hard-coded config overrides. Applied after reading the config file from [Self::config_file], and
    /// thus overrides its values
    pub config_overrides: HashMap<String, crate::config::toml::Value>,
}

impl Default for InitRuntimeConfig {
    fn default() -> Self {
        Self {
            config_file: Some(PathBuf::from("wutengine.toml")),
            config_overrides: Default::default(),
        }
    }
}

/// Starts and runs the WutEngine runtime. MUST be called from the main thread
///
/// Can only be called once per process
pub fn run(
    initial_systems: SystemManifest,
    config: InitRuntimeConfig,
    post_start: Option<Box<dyn FnOnce()>>,
) -> Result<(), Box<RuntimeStartErr>> {
    if WUTENGINE_RUNNING.swap(true, Ordering::AcqRel) {
        return Err(Box::new(RuntimeStartErr::AlreadyRunning));
    }

    wutengine_util::set_cur_thread_as_main_thread();

    crate::event::init();

    // Initialize the config manager early, so all other managers and engine systems
    // can read from it to configure themselves
    let mut logs = crate::config::init_and_load(config.config_file.as_deref());

    for (key, val) in config.config_overrides {
        if let Err(e) = crate::config::set_raw(&key, val) {
            logs.push((
                log::Level::Warn,
                format!("Failed to set config override `{key}` due to error: {e}"),
            ));
        }
    }

    // Then the logger, so we can receive proper feedback
    initialize_logger(logs);

    log::info!("Starting WutEngine");

    #[cfg(feature = "development_overlay")]
    {
        use crate::development_overlay::ConfigOverlay;

        crate::development_overlay::init();
        crate::development_overlay::add_development_overlay_window(ConfigOverlay::default());
        crate::development_overlay::add_development_overlay_window(
            crate::profiling::development_overlay::ProfilingOverlay::default(),
        );
    }

    let mut runtime = Runtime {
        frame_pacer: window::pacer::FramePacer::default(),
        initialization_data: Some(Box::new(InitializationData {
            post_start_callback: post_start,
        })),
        entity_manager: entity::initialize(),
        systems: system::SystemManager::new(),
        draw_commands: graphics::initialize_command_queue(),
    };

    runtime.systems.build_schedule(initial_systems);

    log::debug!("Final schedule:\n{}", runtime.systems.dump());

    window::manager::init();
    input::init();
    physics2d::init();
    physics3d::init();
    audio::init();
    world::init();

    let event_loop = winit::event_loop::EventLoop::<WinitEvent>::with_user_event()
        .build()
        .map_err(|e| Box::new(e.into()))?;

    let event_loop_proxy = event_loop.create_proxy();

    InitOnce::init(&EVENT_LOOP_PROXY, event_loop_proxy);

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    event_loop
        .run_app(&mut runtime)
        .map_err(|e| Box::new(e.into()))?;

    Ok(())
}

fn initialize_logger(queued_messages: Vec<(log::Level, String)>) {
    let default_internal_level = crate::config::try_get("wutengine.log.default_internal_level")
        .unwrap_or(if cfg!(debug_assertions) {
            log::LevelFilter::Info
        } else {
            log::LevelFilter::Warn
        });

    let default_external_level = crate::config::try_get("wutengine.log.default_external_level")
        .unwrap_or(if cfg!(debug_assertions) {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        });

    let logger = wutengine_logger::ModuleLogger::new(
        wutengine_logger::LoggerFactory {
            create_logger: Box::new(log_factory_fn),
        },
        default_internal_level,
        default_external_level,
    );

    let result = log::set_boxed_logger(Box::new(logger));

    if result.is_ok() {
        // `set_boxed_logger` returns an error if the global logger was already
        // initialized. Therefore, if we _dont_ get an error, we can assume that
        // we have to do all the initialization.

        // We set the max level to the static max level, because the individual
        // subsystem loggers can have their own individual filters
        log::set_max_level(log::STATIC_MAX_LEVEL);
    } else {
        // Logger was already initialized, so we can safely log here
        log::info!("A logger was already initialized, so not setting WutEngine logger");
    }

    for (level, msg) in queued_messages {
        log::log!(level, "Queued log: {msg}");
    }
}

//TODO: Can remove option from target?
fn log_factory_fn(filter: log::LevelFilter, target: &str, is_internal: bool) -> Arc<dyn log::Log> {
    _ = target;

    Arc::from(simplelog::TermLogger::new(
        filter,
        simplelog::ConfigBuilder::new()
            .set_location_level(if is_internal {
                log::LevelFilter::Trace
            } else {
                log::LevelFilter::Debug
            })
            .set_target_level(log::LevelFilter::Error)
            .build(),
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Auto,
    ) as Box<dyn log::Log>)
}

impl Runtime {
    fn run_frame(&mut self) {
        // Open another scope so that we're not in the middle of a profiling scope when we call `finish_frame`
        {
            profiling::function_scope!();

            assert_main_thread!();

            // Toggle profiling scopes
            crate::profiling::change_scope_active_status();

            // Run any events sent after the previous frame ended
            crate::event::handle_events();

            // We wait for the rendering target to become available in the beginning of the frame,
            // because then if we block on vsync or similar the simulation will not be out of date
            let surfaces = window::manager::get_surface_textures();

            input::gamepad::poll_for_events();

            let dev_overlay_output = Self::prepare_development_overlay(&surfaces);

            self.run_systems_and_logic();

            self.render_all_windows(&surfaces);

            if let Some((dev_overlay_window, dev_overlay_ready_chan)) = dev_overlay_output {
                #[cfg(not(feature = "development_overlay"))]
                {
                    _ = (dev_overlay_window, dev_overlay_ready_chan);

                    unsafe {
                        wutengine_util::unreachable_dbg!();
                    }
                }

                #[cfg(feature = "development_overlay")]
                {
                    use wutengine_graphics::wgpu;

                    let mut overlay_encoder = graphics::device().create_command_encoder(
                        &wgpu::CommandEncoderDescriptor {
                            label: Some("Development overlay command encoder"),
                        },
                    );

                    {
                        profiling::scope!("Wait for dev overlay");
                        // Wait for the development overlay to be ready, and actually render it to the surface
                        dev_overlay_ready_chan.recv().unwrap();
                    }

                    let (_, dev_overlay_surface) = surfaces
                        .iter()
                        .find(|(win, _)| *win == dev_overlay_window)
                        .unwrap();

                    if crate::development_overlay::render_overlay(
                        &dev_overlay_surface.texture,
                        &mut overlay_encoder,
                    ) {
                        graphics::queue().submit([overlay_encoder.finish()]);
                    }
                }
            }

            for (_, surface) in surfaces {
                surface.present();
            }

            input::end_frame();

            self.frame_pacer.frame_rendered();
            self.frame_pacer.wait_for_limit();
        }

        profiling::finish_frame!();
    }

    fn run_systems_and_logic(&self) {
        profiling::function_scope!();

        let num_fixed_updates = time::update_frame(Instant::now());

        for _ in 0..num_fixed_updates {
            self.run_phase_systems(Phase::FixedUpdate);

            {
                profiling::scope!("Run physics");

                rayon::join(
                    || {
                        physics2d::step(time::fixed_delta());
                    },
                    || {
                        physics3d::step(time::fixed_delta());
                    },
                );
            }

            time::update_fixed();
        }

        self.run_phase_systems(Phase::Update);

        self.run_phase_systems(Phase::LateUpdate);

        self.run_phase_systems(Phase::PreRender);
    }

    fn run_phase_systems(&self, phase: Phase) {
        profiling::function_scope!(phase.str());

        self.systems
            .run_systems_for_phase(phase, &world::get_world());

        entity::process_changes(&mut world::get_world_mut(), &self.entity_manager);

        crate::event::handle_events();
    }

    fn render_all_windows(&self, surfaces: &[(Window, wgpu::SurfaceTexture)]) {
        profiling::function_scope!();

        // Gather all submitted draw commands
        let draw_commands = {
            profiling::scope!("Gather draw commands");

            self.draw_commands.try_iter().collect::<Vec<_>>()
        };

        log::trace!("Gathered {} draw commands this frame", draw_commands.len());

        let mut world = world::get_world_mut();

        // Collect all global passes first
        let passes = world
            .ecs
            .query::<&GlobalRenderPass>()
            .iter()
            .filter_map(|global_pass| global_pass.pass.clone())
            .collect::<Vec<_>>();

        // Render all cameras, giving each camera its own rendering thread
        // TODO: Only render cameras for which the surface is available
        let mut buffers: Vec<_> = world
            .ecs
            .query_mut::<&mut Camera>()
            .into_iter()
            .par_bridge()
            .filter_map(|camera| {
                profiling::scope!("Collect camera command buffer");

                Self::render_camera(camera, &passes, &draw_commands).map(|encoder| encoder.finish())
            })
            .collect();

        // Now that all the rendering is finished, we have the cameras blit their
        // rendered contents onto their main target surface
        let mut blit_encoder =
            graphics::device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Camera Blit Command Encoder"),
            });

        blit_encoder.push_debug_group("Blitting cameras to main surfaces");

        for camera in world.ecs.query_mut::<&mut Camera>() {
            camera.blit_to_target(&mut blit_encoder, surfaces);
        }

        blit_encoder.pop_debug_group();

        buffers.push(blit_encoder.finish());

        window::manager::pre_present_notify(surfaces.iter().map(|(win, _)| win));

        graphics::queue().submit(buffers);
    }

    fn render_camera(
        camera: &mut Camera,
        passes: &[RenderPassInfo<Camera, DrawCommand>],
        draw_commands: &[DrawCommand],
    ) -> Option<wgpu::CommandEncoder> {
        profiling::function_scope!();

        if camera.get_render_target().is_none() {
            log::debug!(
                "Not rendering camera {} because it has no target",
                camera.get_id()
            );
            return None;
        }

        sync_camera_passes(camera, passes);

        let mut encoder =
            graphics::device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(&format!("Camera {} command encoder", camera.get_id())),
            });

        {
            profiling::scope!("Execute render passes");

            // Take the passes out for memory safety...
            let mut passes = core::mem::take(&mut camera.render_passes);

            for pass in passes.iter_mut() {
                profiling::scope!(format!("Execute render pass \n{}\n", pass.name));

                encoder.push_debug_group(pass.name);

                pass.pass.execute(&mut encoder, camera, draw_commands);

                encoder.pop_debug_group();
            }
            // ...and put the passes back
            camera.render_passes = passes;
        }

        Some(encoder)
    }

    fn prepare_development_overlay(
        surfaces: &[(Window, wgpu::SurfaceTexture)],
    ) -> Option<(Window, std::sync::mpsc::Receiver<()>)> {
        #[cfg(feature = "development_overlay")]
        {
            use wutengine_development_overlay::wutengine_egui;

            let main_surface = surfaces.iter().find(|(win, _)| win.is_primary());

            if let Some((window, surface_tex)) = main_surface {
                let egui_window_info = wutengine_egui::EguiWindowInfo {
                    focused: window.is_focused(),
                    occluded: window.is_occluded(),
                    minimized: window.is_minimized(),
                    maximized: window.is_maximized(),
                };

                let ready_channel = crate::development_overlay::run_overlay_logic(
                    input::WindowIdentifier::from(*window),
                    egui_window_info,
                    (
                        surface_tex.texture.size().width,
                        surface_tex.texture.size().height,
                    ),
                    window.get_scale_factor() as f32,
                    time::unscaled_time64(),
                );

                Some((*window, ready_channel))
            } else {
                None
            }
        }

        #[cfg(not(feature = "development_overlay"))]
        {
            _ = surfaces;
            None
        }
    }
}

/// Synchronize the passes on the camera with the passes in `passes`, deleting
/// any passes not in `passes`, and adding missing onces
pub fn sync_camera_passes(camera: &mut Camera, passes: &[RenderPassInfo<Camera, DrawCommand>]) {
    profiling::scope!("Synchronize passes");

    let cam_id = camera.get_id();

    // Remove all passes not present in the global runtime
    camera.render_passes.retain(|camera_pass| {
        let should_keep = passes
            .iter()
            .any(|runtime_pass| runtime_pass.type_id == camera_pass.type_id);

        if !should_keep {
            log::debug!("Removing pass {} from camera {}", camera_pass.name, cam_id);
        }

        should_keep
    });

    // Add passes present in the runtime, but missing in the camera
    let mut passes_added = false;

    for pass in passes {
        if !camera
            .render_passes
            .iter()
            .any(|camera_pass| camera_pass.type_id == pass.type_id)
        {
            log::debug!("Adding pass {} to camera {}", pass.name, cam_id);

            camera.render_passes.push(CameraRenderPass {
                type_id: pass.type_id,
                name: pass.name,
                order: pass.order,
                pass: (pass.constructor)(),
            });

            passes_added = true;
        }
    }

    if passes_added {
        camera.render_passes.sort_by_key(|p| p.order);
    }
}

/// Requests that the WutEngine runtime stops cleanly.
/// This usually happens somewhere before the next frame.
pub fn exit() {
    if !WUTENGINE_RUNNING.load(Ordering::Acquire) {
        log::error!("WutEngine runtime is not running. Cannot request exit");
        return;
    }

    log::info!("Runtime exit requested.");

    if EVENT_LOOP_PROXY
        .send_event(WinitEvent::RuntimeExitRequested)
        .is_err()
    {
        log::error!("Failed to send runtime exit event because the event loop was already closed");
    }
}
