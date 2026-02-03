//! Implements the [winit::application::ApplicationHandler] interface for [crate::runtime::Runtime],
//! so that its execution can be driven by [winit]

use std::sync::Arc;
use std::time::Instant;

use smallvec::SmallVec;
use wgpu::wgt::{CommandEncoderDescriptor, TextureViewDescriptor};
use wgpu::{Color, Operations, RenderPassColorAttachment, RenderPassDescriptor};

use crate::system::Phase;
use crate::window::{Window, WindowConfig};
use crate::{entity, graphics, time, window, world};

use super::Runtime;

/// An event sent to the main WutEngine [Runtime], to be handled by [winit::application::ApplicationHandler::user_event].
///
/// This is meant for events that should be handled on the main thread
#[derive(Debug)]
pub(crate) enum WinitEvent {
    /// The creation of a new window with the given ID and config was requested
    NewWindowRequested(Window, WindowConfig),

    /// A window was requested to be closed
    CloseWindow(Window),

    /// Update the icon of a window
    UpdateIcon(Window, winit::window::Icon),
}

impl winit::application::ApplicationHandler<WinitEvent> for Runtime {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        _ = event_loop;

        let Some(mut post_init) = self.initialization_data.take() else {
            // Already initialized
            return;
        };

        profiling::scope!("Initialize");

        log::info!("Winit resume received, running runtime initialization code");

        if !graphics::initialize_graphics_context() {
            // We could not initialize the graphics context, so quit
            event_loop.exit();
        }

        // Initialize the time manager later here, right before the runtime starts running frames
        time::init();

        // Must be called last, so we know the engine setup is done
        if let Some(post_init_callback) = post_init.post_start_callback.take() {
            post_init_callback();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        native_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(id) = window::manager::find_id(native_id) else {
            log::warn!(
                "Could not find WutEngine window for native ID: {}",
                u64::from(native_id)
            );
            return;
        };

        match event {
            winit::event::WindowEvent::CloseRequested => {
                profiling::scope!("Close Requested");

                event_loop.exit();
            }
            winit::event::WindowEvent::Resized(_) => {
                profiling::scope!("Resized");

                window::manager::refresh_cached_info(&id);

                if cfg!(windows) {
                    // Workaround for https://github.com/rust-windowing/winit/issues/3272
                    // The frame still freezes, but at least the whole window is redrawn once
                    // to make sure it's filled
                    self.about_to_wait(event_loop);
                }
            }
            _ => {}
        }
    }

    fn new_events(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        let _ = (event_loop, cause);
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: WinitEvent) {
        match event {
            WinitEvent::NewWindowRequested(window_id, window_config) => {
                profiling::scope!("New Window Requested");

                log::debug!("Handling window creation request for window {window_id}");

                let native = match event_loop.create_window(window_config.into()) {
                    Ok(native) => Arc::new(native),
                    Err(e) => {
                        log::error!("Failed to create native window for window {window_id}: {e}");
                        return;
                    }
                };

                let surface = graphics::instance().create_surface(native.clone()).unwrap();

                window::manager::new_window(window_id, native, surface);
            }
            WinitEvent::CloseWindow(window_id) => {
                profiling::scope!("Close Window");

                log::debug!("Handling close window request for window {window_id}");

                window::manager::close_window(window_id);
            }
            WinitEvent::UpdateIcon(window_id, icon) => {
                profiling::scope!("Update Icon");

                log::debug!("Handling icon update request for window {window_id}");

                window::manager::set_icon(window_id, icon);
            }
        }
    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let _ = (event_loop, device_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        profiling::finish_frame!();
        profiling::scope!("about_to_wait");

        let _ = event_loop;

        self.run_frame_logic();

        Self::render_all_windows();
    }

    fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        profiling::scope!("Exiting");

        _ = event_loop;

        log::info!("Exiting WutEngine");

        log::logger().flush();
    }

    fn memory_warning(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }
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
    }

    fn run_phase_systems(&self, phase: Phase) {
        profiling::function_scope!(phase.str());

        self.systems
            .run_systems_for_phase(phase, &world::get_world());

        entity::process_changes(&mut world::get_world_mut(), &self.entity_manager);
    }

    fn render_all_windows() {
        profiling::function_scope!();

        let mut buffers = SmallVec::<[_; 4]>::new_const();
        let mut textures = SmallVec::<[_; 4]>::new_const();

        window::manager::with_locked_surfaces(|surfaces| {
            for (_window_id, surface) in surfaces {
                let surface_texture = surface.get_current_texture().unwrap();
                let tex = &surface_texture.texture;
                let view = tex.create_view(&TextureViewDescriptor::default());

                let mut encoder = graphics::device()
                    .create_command_encoder(&CommandEncoderDescriptor { label: None });

                let _pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: Operations {
                            load: wgpu::LoadOp::Clear(Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });

                drop(_pass);

                buffers.push(encoder.finish());
                textures.push(surface_texture);
            }

            graphics::queue().submit(buffers);

            for surface in textures {
                surface.present();
            }
        });
    }
}
