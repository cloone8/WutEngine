//! Implements the [winit::application::ApplicationHandler] interface for [crate::runtime::Runtime],
//! so that its execution can be driven by [winit]

use crate::graphics;
use crate::window::{WindowConfig, WindowId};

use super::Runtime;

/// An event sent to the main WutEngine [Runtime], to be handled by [winit::application::ApplicationHandler::user_event].
///
/// This is meant for events that should be handled on the main thread
#[derive(Debug)]
pub(crate) enum WinitEvent {
    /// The creation of a new window with the given ID and config was requested
    NewWindowRequested(WindowId, WindowConfig),

    /// A window was requested to be closed
    CloseWindow(WindowId),

    /// Update the icon of a window
    UpdateIcon(WindowId, winit::window::Icon),
}

impl winit::application::ApplicationHandler<WinitEvent> for Runtime {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        _ = event_loop;

        let Some(mut post_init) = self.initialization_data.take() else {
            // Already initialized
            return;
        };

        log::info!("Winit resume received, running runtime initialization code");

        if !graphics::initialize_graphics_context() {
            // We could not initialize the graphics context, so quit
            event_loop.exit();
        }

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
        let Some(id) = self.windows.find_id(native_id) else {
            log::warn!(
                "Could not find WutEngine window for native ID: {}",
                u64::from(native_id)
            );
            return;
        };

        match event {
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::Resized(_) => {
                self.windows.refresh_cached_info(&id);
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
                log::debug!("Handling window creation request for window {window_id}");

                let native = match event_loop.create_window(window_config.into()) {
                    Ok(native) => native,
                    Err(e) => {
                        log::error!("Failed to create native window for window {window_id}: {e}");
                        return;
                    }
                };
                self.windows.new_window(window_id, native);
            }
            WinitEvent::CloseWindow(window_id) => {
                log::debug!("Handling close window request for window {window_id}");

                self.windows.close_window(window_id);
            }
            WinitEvent::UpdateIcon(window_id, icon) => {
                log::debug!("Handling icon update request for window {window_id}");

                self.windows.set_icon(window_id, icon);
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
        let _ = event_loop;
    }

    fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        _ = event_loop;

        log::info!("Exiting WutEngine");

        log::logger().flush();
    }

    fn memory_warning(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }
}
