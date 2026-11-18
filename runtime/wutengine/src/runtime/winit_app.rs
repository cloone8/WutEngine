use winit::application::ApplicationHandler;
use winit::event::DeviceEvent;
use winit::event::DeviceId;
use winit::event::StartCause;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use crate::executor;
use crate::log;
use crate::runtime::startup;

pub(crate) struct WutEngineApp {
    phase: AppPhase,
}

impl WutEngineApp {
    pub(crate) fn new() -> Self {
        Self {
            phase: AppPhase::NotStarted,
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
        if self.phase == AppPhase::NotStarted {
            log::info!("Beginning WutEngine main startup");

            let (instance, main_window, main_window_surface) = startup::startup_sync(event_loop);

            cfg_select! {
                target_arch = "wasm32" => {
                    startup::driver::start(startup::startup());
                    self.phase = AppPhase::Starting;
                }
                _ => {
                    executor::block_on(startup::startup_async(instance, &main_window_surface));
                    self.phase = AppPhase::Running;
                }
            }

            return;
        }

        //TODO: Recreate surfaces
        todo!("Recreate surfaces");
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
