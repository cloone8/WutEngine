//! The main WutEngine runtime, responsible for the application lifecycle

use crate::builtins::components::rendering::ActiveCameraRenderPass;
use crate::builtins::components::rendering::Camera;
use crate::builtins::components::rendering::CameraRenderPass;
use crate::builtins::components::rendering::OverlayRenderPass;
use crate::entity;
use crate::entity::EntityManager;
use crate::graphics;
use crate::graphics::DrawCommand;
use crate::graphics::RenderPassInfo;
use crate::input;
use crate::physics2d;
use crate::physics3d;
use crate::system::Phase;
use crate::system::SystemManager;
use crate::time;
use crate::window;
use crate::window::Window;
use crate::world;
use core::any::TypeId;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::time::Instant;
use wutengine_graphics::label;
use wutengine_graphics::renderpass::RenderPass;
use wutengine_graphics::wgpu;
use wutengine_util::InitOnce;
use wutengine_util::assert_main_thread;

use rayon::prelude::*;

mod init;
mod system_builder;
mod winit_app;

pub use init::*;

pub use system_builder::*;

pub(crate) use winit_app::MainThreadEvent;

static EVENT_LOOP_PROXY: InitOnce<winit::event_loop::EventLoopProxy<MainThreadEvent>> =
    InitOnce::new();
static WUTENGINE_RUNNING: AtomicBool = AtomicBool::new(false);

/// Notifies the main [winit] event loop of a given event.
///
/// If the loop was already closed, does nothing and logs an error
pub(crate) fn send_to_main_thread(event: MainThreadEvent) {
    if let Err(e) = EVENT_LOOP_PROXY.send_event(event) {
        log::error!(
            "Failed to notify event loop of event {:#?} because it was already closed",
            e.0
        );
    }
}

/// The main WutEngine runtime
pub(crate) struct Runtime {
    /// Used for frame pacing and FPS limiting
    frame_pacer: window::pacer::FramePacer,

    /// Set to [None] if the `resumed` event was sent by [winit]
    initialization_data: Option<Box<InitializationData>>,

    /// The entity manager. Spawns entities and components
    entity_manager: EntityManager,

    /// Contains the systems and their schedule
    systems: SystemManager,

    /// The receiving end of the graphics command queue
    draw_commands: Receiver<crate::graphics::DrawCommand>,

    overlay_passes: Vec<ActiveOverlayRenderPass>,

    /// Whether to always request a redraw when the event loop is going to sleep
    always_redraw: bool,
}

///TODO: Combine with [ActiveCameraRenderPass] with a generic?
#[derive(derive_more::Debug)]
struct ActiveOverlayRenderPass {
    /// The type of the pass
    pub(crate) type_id: TypeId,

    /// The name of the pass
    pub(crate) name: &'static str,

    /// The order of the pass relative to other passes
    pub(crate) order: u64,

    /// The pass itself
    #[debug(skip)]
    pub(crate) pass: Box<dyn for<'a> RenderPass<(Window, wgpu::Texture), ()>>,
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

            Self::prepare_development_overlay(&surfaces);

            self.run_systems_and_logic();

            self.render_all_windows(&surfaces);

            for (_, surface) in surfaces {
                surface.present();
            }

            input::end_frame();

            self.frame_pacer.frame_rendered();
            self.frame_pacer.wait_for_limit();
        }

        profiling::finish_frame!();
    }

    fn run_systems_and_logic(&mut self) {
        profiling::function_scope!();

        let num_fixed_updates = time::update_frame(Instant::now());

        for _ in 0..num_fixed_updates {
            self.run_physics_pipeline();
        }

        self.run_phase_systems(Phase::Update);

        self.run_phase_systems(Phase::LateUpdate);

        self.run_phase_systems(Phase::PreRender);
    }

    fn run_physics_pipeline(&mut self) {
        profiling::function_scope!();

        self.run_phase_systems(Phase::FixedUpdate);

        Self::write_physics_state();

        {
            profiling::scope!("Run physics step");

            rayon::join(
                || {
                    physics2d::step(time::fixed_delta());
                },
                || {
                    physics3d::step(time::fixed_delta());
                },
            );
        }

        Self::read_physics_state();

        time::update_fixed();
    }

    fn write_physics_state() {
        use crate::builtins::components::Transform;
        use crate::builtins::components::physics::*;

        profiling::function_scope!();

        rayon::join(
            || {
                let world = world::get_world();

                let mut query = world
                    .ecs
                    .query::<(&mut ColliderSet2D, Option<&Transform>)>();

                physics2d::update_physics_world(|updater| {
                    for (set2d, xform) in query.iter() {
                        set2d.sync_to_physics_world(xform, updater);
                    }
                });
            },
            || {
                let world = world::get_world();

                let mut query = world
                    .ecs
                    .query::<(&mut ColliderSet3D, Option<&Transform>)>();

                physics3d::update_physics_world(|updater| {
                    for (set3d, xform) in query.iter() {
                        set3d.sync_to_physics_world(xform, updater);
                    }
                });
            },
        );
    }
    fn read_physics_state() {
        use crate::builtins::components::Transform;
        use crate::builtins::components::physics::*;

        profiling::function_scope!();

        let mut world = world::get_world_mut();

        {
            profiling::scope!("Read 2D state");
            let query = world
                .ecs
                .query_mut::<(&mut ColliderSet2D, Option<&mut Transform>)>();

            for _val in query {}
        }

        {
            profiling::scope!("Read 3D state");
            let query = world
                .ecs
                .query_mut::<(&mut ColliderSet3D, Option<&mut Transform>)>();

            for _val in query {}
        }
    }

    fn run_phase_systems(&mut self, phase: Phase) {
        profiling::function_scope!(phase.str());

        self.systems.update_schedule();

        self.systems
            .run_systems_for_phase(phase, &world::get_world());

        entity::process_changes(&mut world::get_world_mut(), &self.entity_manager);

        crate::event::handle_events();
    }

    fn render_all_windows(&mut self, surfaces: &[(Window, wgpu::SurfaceTexture)]) {
        profiling::function_scope!();

        // Gather all submitted draw commands
        let draw_commands = {
            profiling::scope!("Gather draw commands");

            self.draw_commands.try_iter().collect::<Vec<_>>()
        };

        log::trace!("Gathered {} draw commands this frame", draw_commands.len());

        let mut world = world::get_world_mut();

        // Collect all global passes first
        let camera_passes = world
            .ecs
            .query_mut::<&CameraRenderPass>()
            .into_iter()
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

                Self::render_camera(camera, &camera_passes, &draw_commands)
                    .map(|encoder| encoder.finish())
            })
            .collect();

        // Now that all the rendering is finished, we have the cameras blit their
        // rendered contents onto their main target surface
        let mut blit_encoder =
            graphics::device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: label!("Camera Blit Command Encoder"),
            });

        blit_encoder.push_debug_group("Blitting cameras to main surfaces");

        for camera in world.ecs.query_mut::<&mut Camera>() {
            camera.blit_to_target(&mut blit_encoder, surfaces);
        }

        blit_encoder.pop_debug_group();

        buffers.push(blit_encoder.finish());

        graphics::queue().submit(buffers);

        // Render all the overlays now
        let overlay_passes = world
            .ecs
            .query_mut::<&OverlayRenderPass>()
            .into_iter()
            .filter_map(|overlay_pass| overlay_pass.pass.clone())
            .collect::<Vec<_>>();

        sync_overlay_passes(&mut self.overlay_passes, &overlay_passes);

        let mut overlay_command_encoder =
            graphics::device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: label!("Overlay Command Encoder"),
            });

        overlay_command_encoder.push_debug_group("Render overlay passes");

        for overlay_pass in self.overlay_passes.iter_mut() {
            for (window, surface_texture) in surfaces {
                overlay_command_encoder.push_debug_group(
                    format!("Overlay {} window {}", overlay_pass.name, *window).as_str(),
                );

                overlay_pass.pass.execute(
                    &mut overlay_command_encoder,
                    &(*window, surface_texture.texture.clone()),
                    &(),
                );

                overlay_command_encoder.pop_debug_group();
            }
        }

        overlay_command_encoder.pop_debug_group();

        graphics::queue().submit([overlay_command_encoder.finish()]);

        window::manager::pre_present_notify(surfaces.iter().map(|(win, _)| win));
    }

    fn render_camera(
        camera: &mut Camera,
        passes: &[RenderPassInfo<Camera, [DrawCommand]>],
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
                label: label!("Camera {} command encoder", camera.get_id()),
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

    fn prepare_development_overlay(surfaces: &[(Window, wgpu::SurfaceTexture)]) {
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

                crate::development_overlay::run_overlay_logic(
                    input::WindowIdentifier::from(*window),
                    egui_window_info,
                    (
                        surface_tex.texture.size().width,
                        surface_tex.texture.size().height,
                    ),
                    window.get_scale_factor() as f32,
                );
            }
        }

        #[cfg(not(feature = "development_overlay"))]
        {
            _ = surfaces;
        }
    }
}

/// Synchronize the passes on the camera with the passes in `passes`, deleting
/// any passes not in `passes`, and adding missing onces
fn sync_camera_passes(camera: &mut Camera, passes: &[RenderPassInfo<Camera, [DrawCommand]>]) {
    profiling::function_scope!();

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

            camera.render_passes.push(ActiveCameraRenderPass {
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

fn sync_overlay_passes(
    active_passes: &mut Vec<ActiveOverlayRenderPass>,
    passes: &[RenderPassInfo<(Window, wgpu::Texture), ()>],
) {
    profiling::function_scope!();

    // Remove all passes not present in the global runtime
    active_passes.retain(|active_pass| {
        let should_keep = passes
            .iter()
            .any(|runtime_pass| runtime_pass.type_id == active_pass.type_id);

        if !should_keep {
            log::debug!("Removing overlay pass {}", active_pass.name);
        }

        should_keep
    });

    // Add passes present in the runtime, but missing in the camera
    let mut passes_added = false;

    for pass in passes {
        if !active_passes
            .iter()
            .any(|active_pass| active_pass.type_id == pass.type_id)
        {
            log::debug!("Adding overlay pass {}", pass.name);

            active_passes.push(ActiveOverlayRenderPass {
                type_id: pass.type_id,
                name: pass.name,
                order: pass.order,
                pass: (pass.constructor)(),
            });

            passes_added = true;
        }
    }

    if passes_added {
        active_passes.sort_by_key(|p| p.order);
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
        .send_event(MainThreadEvent::RuntimeExitRequested)
        .is_err()
    {
        log::error!("Failed to send runtime exit event because the event loop was already closed");
    }
}
