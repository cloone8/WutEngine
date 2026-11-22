use core::time::Duration;
use std::collections::HashMap;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::DeviceEvent;
use winit::event::DeviceId;
use winit::event::StartCause;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window as WinitWindow;
use winit::window::WindowAttributes;
use winit::window::WindowId;
use wutengine_renderer::surfaces;
use wutengine_renderer::surfaces::SurfaceId;
use wutengine_renderer::surfaces::SurfaceType;

use crate::executor;
use crate::log;
use crate::runtime::frame::FrameState;
use crate::runtime::startup;
use crate::time;
use crate::window;
use crate::window::Window;

mod window_events;

#[derive(Debug)]
pub(crate) struct WutEngineApp {
    phase: AppPhase,
    window_map: HashMap<WindowId, Window>,
    surface_map: HashMap<Window, SurfaceId>,
}

impl WutEngineApp {
    pub(crate) fn new() -> Self {
        Self {
            phase: AppPhase::NotStarted,
            window_map: HashMap::default(),
            surface_map: HashMap::default(),
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
enum AppPhase {
    NotStarted,
    #[cfg(target_arch = "wasm32")]
    Starting,
    Running {
        frame_state: FrameState,
    },
}

impl ApplicationHandler for WutEngineApp {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        _ = event_loop;

        if matches!(self.phase, AppPhase::NotStarted) {
            log::info!("Beginning WutEngine main startup");

            let main_window = startup::startup_sync();

            self.initialize_pending_windows(event_loop);

            let main_surface = self
                .surface_map
                .get(&main_window)
                .copied()
                .expect("Missing main window surface");

            cfg_select! {
                target_arch = "wasm32" => {
                    startup::driver::start(startup::startup());
                    self.phase = AppPhase::Starting;
                }
                _ => {
                    executor::block_on(startup::startup_async(main_surface));
                    self.phase = AppPhase::Running {
                        frame_state: FrameState::new(Duration::from_secs_f64(1.0 / 60.0)), // TODO:
                                                                                           // Make
                                                                                           // configurable?
                    }
                }
            }

            return;
        }

        //TODO: Recreate surfaces
        log::error_once!("Recreating surfaces not yet supported");
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        profiling::function_scope!();

        let Some(window) = self.window_map.get(&window_id).copied() else {
            log::warn!("Got window event for unknown window (winit ID: {window_id:?})");
            return;
        };

        let surface = self.surface_map.get(&window).copied();

        match event {
            WindowEvent::ScaleFactorChanged { .. } => {
                Self::window_scale_factor_changed(window, &event);
            }
            WindowEvent::SurfaceResized(_) => {
                Self::window_surface_resized(window, surface.expect("Missing surface"), &event);
            }
            WindowEvent::CloseRequested => {
                Self::window_close_requested(event_loop, window);
            }
            WindowEvent::Destroyed => {
                self.window_destroyed(
                    window_id,
                    window,
                    surface.expect("Missing surface for window"),
                );
            }
            WindowEvent::RedrawRequested => {
                self.window_redraw_requested(window);
            }
            _ => {}
        }
    }

    fn new_events(&mut self, event_loop: &dyn ActiveEventLoop, cause: StartCause) {
        let _ = (event_loop, cause);
    }

    fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) {
        let _ = event_loop;
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        let _ = event_loop;
    }

    fn device_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        device_id: Option<DeviceId>,
        event: DeviceEvent,
    ) {
        let _ = (event_loop, device_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.initialize_pending_windows(event_loop);

        // Both starts the first frame, and requests redraws for any new windows
        // and windows whose stale redraw requests were dropped
        window::request_redraws(time::frame_num());
    }

    fn suspended(&mut self, event_loop: &dyn ActiveEventLoop) {
        let _ = event_loop;
    }

    fn memory_warning(&mut self, event_loop: &dyn ActiveEventLoop) {
        let _ = event_loop;
    }

    fn destroy_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let _ = event_loop;
        //TODO: Destroy all surfaces here
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    fn macos_handler(
        &mut self,
    ) -> Option<&mut dyn winit::application::macos::ApplicationHandlerExtMacOS> {
        None
    }
}

impl WutEngineApp {
    /// Creates native windows and surfaces for all pending windows
    fn initialize_pending_windows(&mut self, el: &dyn ActiveEventLoop) {
        profiling::function_scope!();

        window::initialize_windows(|window, settings| {
            log::debug!("Initializing window {window}");

            let native_window: Arc<dyn WinitWindow> = Arc::from(
                el.create_window(WindowAttributes::from(settings))
                    .expect("Failed to initialize window"),
            );

            let prev = self.window_map.insert(native_window.id(), window);

            assert!(
                prev.is_none(),
                "Window with native window ID already in map"
            );

            let surface_type = if window.is_main() {
                SurfaceType::Main
            } else {
                SurfaceType::Other
            };

            let surface_size: (u32, u32) = native_window.surface_size().into();

            let surface_id = surfaces::initialize(
                surface_type,
                surface_size,
                Arc::clone(&native_window),
                Some(Box::new(move || {
                    window::pre_present(window);
                })),
            );

            self.surface_map.insert(window, surface_id);

            native_window
        });
    }
}
