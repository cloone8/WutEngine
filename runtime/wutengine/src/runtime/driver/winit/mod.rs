use core::cell::OnceCell;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use core::task::Waker;
use core::time::Duration;
use std::collections::HashMap;
use std::sync::Arc;
use std::task::Wake;

use winit::application::ApplicationHandler;
use winit::dpi::Size;
use winit::event::DeviceEvent;
use winit::event::DeviceId;
use winit::event::StartCause;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::EventLoopProxy;
use winit::window::Window as WinitWindow;
use winit::window::WindowAttributes;
use winit::window::WindowId;
use wutengine_executor::ThreadLocalExecutor;
use wutengine_renderer::surfaces;
use wutengine_renderer::surfaces::SurfaceId;
use wutengine_renderer::surfaces::SurfaceType;

use crate::log;
use crate::runtime;
use crate::runtime::frame::FrameState;
use crate::time;
use crate::window;
use crate::window::Window;
use crate::window::WindowSettings;
use crate::window::WindowSnapshot;
use crate::window::WindowState;

mod conversion;
mod init;
mod window_events;

#[derive(Debug)]
pub(crate) struct WutEngineApp {
    phase: AppPhase,
    window_map: HashMap<WindowId, Window>,
    native_windows: HashMap<Window, WindowData>,
    main_thread_executor: ThreadLocalExecutor,
    executor_signal: OnceCell<Waker>,
}

impl WutEngineApp {
    pub(crate) fn new() -> Self {
        Self {
            phase: AppPhase::NotStarted,
            window_map: HashMap::default(),
            native_windows: HashMap::default(),
            main_thread_executor: ThreadLocalExecutor::new(),
            executor_signal: OnceCell::new(),
        }
    }
}

#[derive(Debug)]
struct WindowData {
    native: Arc<dyn WinitWindow>,
    surface: SurfaceId,
    frame_stamp: Option<u64>,
}

#[derive(Debug)]
#[non_exhaustive]
enum AppPhase {
    NotStarted,
    Starting { ready: Arc<AtomicBool> },
    Running { frame_state: FrameState },
}

impl ApplicationHandler for WutEngineApp {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        if matches!(self.phase, AppPhase::NotStarted) {
            log::info!("Beginning windowing application main startup");

            let main_window = init::initialize_renderer_and_main_window();

            self.initialize_pending_windows(event_loop);

            if let WindowState::Error(e) = main_window.state() {
                runtime::fatal_error(format!("Failed to create main window: {e}").as_str());
            }

            let main_surface = self
                .native_windows
                .get(&main_window)
                .map(|win| win.surface)
                .expect("Missing main window surface");

            let ready_signal = Arc::new(AtomicBool::new(false));
            {
                let ready_signal_cloned = Arc::clone(&ready_signal);

                self.main_thread_executor.spawn(async move {
                    init::initialize_main_surface_and_device(main_surface).await;

                    ready_signal_cloned.store(true, Ordering::Release);
                });
            }

            self.phase = AppPhase::Starting {
                ready: ready_signal,
            };

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

        _ = event_loop;

        let Some(window) = self.window_map.get(&window_id).copied() else {
            log::debug!("Got window event for unknown window (winit ID: {window_id:?})");
            return;
        };

        match event {
            WindowEvent::ScaleFactorChanged { .. } => {
                self.window_scale_factor_changed(window, &event);
            }
            WindowEvent::SurfaceResized(_) => {
                self.window_surface_resized(window, &event);
            }
            WindowEvent::CloseRequested => {
                Self::window_close_requested(window);
            }
            WindowEvent::Destroyed => {
                Self::window_destroyed(window);
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
        self.handle_closed_windows();

        // Both starts the first frame, and requests redraws for any new windows
        // and windows whose stale redraw requests were dropped
        self.request_redraws(time::frame_num());

        // Run the event loop
        self.main_thread_executor.run_ready(
            self.executor_signal
                .get_or_init(|| Waker::from(WinitWaker::new(event_loop))),
        );

        if let AppPhase::Starting { ready } = &self.phase
            && ready.load(Ordering::Acquire)
        {
            log::debug!("Async startup ready signal given.");

            self.phase = AppPhase::Running {
                frame_state: FrameState::new(Duration::from_secs_f64(1.0 / 60.0)), // TODO:
                                                                                   // Make
                                                                                   // configurable?
            }
        }
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

            let surface_type = if window.is_main() {
                SurfaceType::Main
            } else {
                SurfaceType::Other
            };

            let window_attributes = window_settings_to_attributes(settings);

            #[cfg(target_arch = "wasm32")]
            let window_attributes =
                crate::platform::web::finish_window_attributes(window, window_attributes);

            let native_window: Arc<dyn WinitWindow> = match el.create_window(window_attributes) {
                Ok(native) => Arc::from(native),
                Err(e) => {
                    log::error!("Failed to create native window: {e}");
                    return Err(Arc::new(e));
                }
            };

            let prev = self.window_map.insert(native_window.id(), window);

            assert!(
                prev.is_none(),
                "Window with native window ID already in map"
            );

            let surface_size: (u32, u32) = native_window.surface_size().into();

            let surface_id = {
                let native_window = Arc::clone(&native_window);

                surfaces::initialize(
                    surface_type,
                    surface_size,
                    Arc::clone(&native_window),
                    Some(Box::new(move || {
                        native_window.pre_present_notify();
                    })),
                )
            };

            let snapshot = create_snapshot_from_native(native_window.as_ref());

            self.native_windows.insert(
                window,
                WindowData {
                    native: native_window,
                    surface: surface_id,
                    frame_stamp: None,
                },
            );

            Ok(snapshot)
        });
    }

    fn handle_closed_windows(&mut self) {
        profiling::function_scope!();

        let closed = window::close_windows();

        for window in closed {
            self.cleanup_closed_window(window);
        }
    }

    fn cleanup_closed_window(&mut self, window: Window) {
        profiling::function_scope!();

        let window_data = self
            .native_windows
            .remove(&window)
            .expect("Missing native window");

        let native_window_id = window_data.native.id();

        let removed = self.window_map.remove(&native_window_id);

        assert_eq!(Some(window), removed, "Invalid window mapping given");

        surfaces::remove(window_data.surface);
    }
}

#[derive(Debug)]
struct WinitWaker {
    proxy: EventLoopProxy,
}

impl WinitWaker {
    fn new(el: &dyn ActiveEventLoop) -> Arc<Self> {
        Arc::new(Self {
            proxy: el.create_proxy(),
        })
    }
}

impl Wake for WinitWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.proxy.wake_up();
    }
}

/// Converts these window settings to a winit [`WindowAttributes`].
/// This adds the general options. Each specific platform adds their own options later with
/// [`platform::finish_window_attributes`](crate::platform::finish_window_attributes)
fn window_settings_to_attributes(settings: &WindowSettings) -> WindowAttributes {
    let mut attrs = WindowAttributes::default();

    attrs.surface_size = Some(Size::Logical(conversion::logical_size_to_winit(
        settings.size,
    )));
    attrs.title.clone_from(&settings.title);

    attrs
}
fn create_snapshot_from_native(native: &dyn WinitWindow) -> WindowSnapshot {
    profiling::function_scope!();

    let pixels = conversion::pixel_size_from_winit(native.surface_size());
    let scale_factor = native.scale_factor();
    let size = pixels.to_logical(scale_factor);

    WindowSnapshot {
        pixels,
        size,
        scale_factor,
    }
}
