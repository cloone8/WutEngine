//! The main WutEngine runtime, responsible for the application lifecycle

use core::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::Instant;

use derive_more::{Display, Error, From};
use winit::error::EventLoopError;

use crate::builtins::components::rendering::Camera;
use crate::builtins::components::rendering::GlobalRenderPass;
use crate::entity::{self, EntityManager};
use crate::graphics::DrawCommand;
use crate::graphics::renderpass::RenderPassInfo;
use crate::input;
use crate::system::{self, Phase, SystemManager};
use crate::util::{self, InitOnce};
use crate::window::{self, Window};
use crate::{graphics, time, world};

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
    systems: SystemManifest,
    config: InitRuntimeConfig,
    post_start: Option<Box<dyn FnOnce()>>,
) -> Result<(), Box<RuntimeStartErr>> {
    if WUTENGINE_RUNNING.swap(true, Ordering::AcqRel) {
        return Err(Box::new(RuntimeStartErr::AlreadyRunning));
    }

    log::info!("Starting WutEngine");

    util::set_cur_thread_as_main_thread();

    let mut runtime = Runtime {
        initialization_data: Some(Box::new(InitializationData {
            post_start_callback: post_start,
        })),
        entity_manager: entity::initialize(),
        systems: system::SystemManager::new(),
        draw_commands: graphics::initialize_command_queue(),
    };

    // Initialize the config manager first, so all other managers and engine systems
    // can read from it to configure themselves
    crate::config::init_and_load(config.config_file.as_deref());

    for (key, val) in config.config_overrides {
        if let Err(e) = crate::config::set_raw(&key, val) {
            log::warn!("Failed to set config override `{key}` due to error: {e}");
        }
    }

    runtime.systems.build_schedule(systems);

    log::debug!("Final schedule:\n{}", runtime.systems.dump());

    window::manager::initialize();
    input::init();
    world::initialize();

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

impl Runtime {
    fn run_frame_logic(&self) {
        profiling::function_scope!();

        let num_fixed_updates = time::update_frame(Instant::now());

        for _ in 0..num_fixed_updates {
            self.run_phase_systems(Phase::FixedUpdate);

            time::update_fixed();
        }

        self.run_phase_systems(Phase::Update);

        self.run_phase_systems(Phase::PreRender);
    }

    fn run_phase_systems(&self, phase: Phase) {
        profiling::function_scope!(phase.str());

        self.systems
            .run_systems_for_phase(phase, &world::get_world());

        entity::process_changes(&mut world::get_world_mut(), &self.entity_manager);
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
        passes: &[RenderPassInfo],
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

        graphics::renderpass::sync_camera_passes(camera, passes);

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
