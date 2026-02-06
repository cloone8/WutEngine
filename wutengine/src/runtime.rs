//! The main WutEngine runtime, responsible for the application lifecycle

use core::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::time::Instant;

use derive_more::{Display, Error, From};
use smallvec::SmallVec;
use wgpu::wgt::CommandEncoderDescriptor;
use winit::error::EventLoopError;

use crate::builtins::components::Camera;
use crate::entity::{self, EntityManager};
use crate::graphics::DrawCommand;
use crate::system::{self, Phase, SystemManager};
use crate::util::InitOnce;
use crate::window::{self, Window};
use crate::world::get_world_mut;
use crate::{graphics, time, world};

mod system_builder;
mod winit_app;

pub use system_builder::*;

pub(crate) use winit_app::WinitEvent;

static EVENT_LOOP_PROXY: InitOnce<winit::event_loop::EventLoopProxy<WinitEvent>> = InitOnce::new();

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

/// Starts and runs the WutEngine runtime. MUST be called from the main thread
///
/// Can only be called once per process
pub fn run(
    systems: SystemManifest,
    post_start: Option<Box<dyn FnOnce()>>,
) -> Result<(), RuntimeStartErr> {
    static WUTENGINE_RUNNING: AtomicBool = AtomicBool::new(false);

    if WUTENGINE_RUNNING.swap(true, Ordering::AcqRel) {
        return Err(RuntimeStartErr::AlreadyRunning);
    }

    log::info!("Starting WutEngine");

    InitOnce::init(&crate::MAIN_THREAD_ID, std::thread::current().id());

    let mut runtime = Runtime {
        initialization_data: Some(Box::new(InitializationData {
            post_start_callback: post_start,
        })),
        entity_manager: entity::initialize(),
        systems: system::SystemManager::new(),
        draw_commands: graphics::initialize_command_queue(),
    };

    runtime.systems.build_schedule(systems);

    log::debug!("Final schedule:\n{}", runtime.systems.dump());

    window::manager::initialize();
    world::initialize();

    let event_loop = winit::event_loop::EventLoop::<WinitEvent>::with_user_event().build()?;
    let event_loop_proxy = event_loop.create_proxy();

    InitOnce::init(&EVENT_LOOP_PROXY, event_loop_proxy);

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    event_loop.run_app(&mut runtime)?;

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

    fn render_all_windows(&self) {
        profiling::function_scope!();

        // Gather all submitted draw commands
        let draw_commands = self.draw_commands.try_iter().collect::<Vec<_>>();

        log::trace!("Gathered {} draw commands this frame", draw_commands.len());

        let mut buffers = SmallVec::<[_; 8]>::new_const();

        let mut world = world::get_world_mut();

        for camera in world.ecs.query_mut::<&mut Camera>() {
            if let Some(encoder) = Self::render_camera(camera, &draw_commands) {
                buffers.push(encoder.finish());
            }
        }

        // Now that all the rendering is finished, we lock the window surfaces and have the cameras blit their
        // rendered contents onto their main target surface
        window::manager::with_locked_surfaces(|surfaces| {
            let mut surface_texture_map = SmallVec::<[_; 4]>::new_const();

            for (window, surface) in surfaces {
                let surface_texture = surface.get_current_texture().unwrap();
                surface_texture_map.push((*window, surface_texture));
            }

            let mut blit_encoder =
                graphics::device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Camera Blit Command Encoder"),
                });

            for camera in world.ecs.query_mut::<&mut Camera>() {
                camera.blit_to_target(&mut blit_encoder, &surface_texture_map);
            }

            buffers.push(blit_encoder.finish());

            graphics::queue().submit(buffers);

            for (_, surface) in surface_texture_map {
                surface.present();
            }
        });
    }

    fn render_camera(
        camera: &mut Camera,
        draw_commands: &[DrawCommand],
    ) -> Option<wgpu::CommandEncoder> {
        let mut encoder =
            graphics::device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Camera Rendering Command Encoder"),
            });

        let Some(render_pass) = camera.begin_pass(&mut encoder) else {
            //
            return None;
        };

        //TODO: Execute draw commands

        drop(render_pass);
        Some(encoder)
    }
}
