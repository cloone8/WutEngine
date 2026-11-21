use std::collections::HashMap;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::DeviceEvent;
use winit::event::DeviceId;
use winit::event::StartCause;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window as WinitWindow;
use winit::window::WindowId;

use crate::executor;
use crate::log;
use crate::renderer::surfaces;
use crate::runtime::startup;
use crate::window;
use crate::window::Window;

#[derive(Debug)]
pub(crate) struct WutEngineApp {
    phase: AppPhase,
    window_map: HashMap<WindowId, Window>,
}

impl WutEngineApp {
    pub(crate) fn new() -> Self {
        Self {
            phase: AppPhase::NotStarted,
            window_map: HashMap::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppPhase {
    NotStarted,
    #[cfg(target_arch = "wasm32")]
    Starting,
    Running,
}

impl ApplicationHandler for WutEngineApp {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        _ = event_loop;

        if self.phase == AppPhase::NotStarted {
            log::info!("Beginning WutEngine main startup");

            let main_window = startup::startup_sync();

            self.initialize_pending_windows(event_loop);

            cfg_select! {
                target_arch = "wasm32" => {
                    startup::driver::start(startup::startup());
                    self.phase = AppPhase::Starting;
                }
                _ => {
                    executor::block_on(startup::startup_async(main_window));
                    self.phase = AppPhase::Running;
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
        _ = event_loop;
        _ = window_id;
        _ = event;

        if event == WindowEvent::CloseRequested {
            event_loop.exit();
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
        let _ = event_loop;
    }

    fn suspended(&mut self, event_loop: &dyn ActiveEventLoop) {
        let _ = event_loop;
    }

    fn memory_warning(&mut self, event_loop: &dyn ActiveEventLoop) {
        let _ = event_loop;
    }

    fn destroy_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let _ = event_loop;
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

        window::initialize_windows(|window, attrs| {
            log::debug!("Initializing window {window}");

            let native_window: Arc<dyn WinitWindow> = Arc::from(
                el.create_window(attrs.clone())
                    .expect("Failed to initialize window"),
            );

            let prev = self.window_map.insert(native_window.id(), window);

            assert!(
                prev.is_none(),
                "Window with native window ID already in map"
            );

            surfaces::initialize_surface(window, Arc::clone(&native_window));

            native_window
        });
    }
}
