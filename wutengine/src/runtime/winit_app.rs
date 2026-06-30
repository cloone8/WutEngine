//! Implements the [winit::application::ApplicationHandler] interface for [crate::runtime::Runtime],
//! so that its execution can be driven by [winit]

use core::time::Duration;

use alloc::sync::Arc;
use wutengine_util_macro::VariantName;

use crate::config;
use crate::graphics;
use crate::input;
use crate::runtime::send_to_main_thread;
use crate::time;
use crate::window;
use crate::window::Window;
use crate::window::WindowConfig;
use crate::window::WindowUpdateEvent;

use super::FrameFrequency;
use super::Runtime;
use super::SystemManifest;

/// An event sent to the main WutEngine [Runtime], to be handled by [winit::application::ApplicationHandler::user_event].
///
/// This is meant for events that should be handled on the main thread
#[derive(derive_more::Debug, VariantName)]
pub(crate) enum MainThreadEvent {
    /// The creation of a new window with the given ID and config was requested
    NewWindowRequested(Window, WindowConfig),

    /// A window was requested to be closed
    CloseWindow(Window),

    /// Update a parameter of a window
    UpdateWindow(Window, WindowUpdateEvent),

    /// Surface should be reconfigured
    ForceSurfaceReconfigure(Window),

    /// Request to add one or more systems to the main system schedule
    AddSystem(SystemManifest),

    /// Run a task on the main thread
    #[debug("RunTask(...)")]
    RunTask(Box<dyn Future<Output = ()> + Send + 'static>),

    /// User requested a redraw
    Redraw,

    /// Someone requested the exit of the runtime through [crate::runtime::exit]
    RuntimeExitRequested,
}

impl winit::application::ApplicationHandler<MainThreadEvent> for Runtime {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        _ = event_loop;

        let Some(mut post_init) = self.initialization_data.take() else {
            // Already initialized
            return;
        };

        profiling::scope!("Initialize");

        window::manager::refresh_displays(event_loop);

        log::info!("Winit resume received, running runtime initialization code");

        if !graphics::initialize_graphics_context() {
            // We could not initialize the graphics context, so quit fast and hard
            log::error!("Doing hard exit because we failed to initialize the graphics context");
            log::logger().flush();
            std::process::exit(808);
        }

        #[cfg(feature = "development_overlay")]
        {
            graphics::dev_overlays::insert_all();
        }

        // Initialize the time manager later here, right before the runtime starts running frames
        time::init();

        if let Some(fps_limit) = config::try_get::<u64>("wutengine.window.fps_limit")
            && fps_limit != 0
        {
            self.frame_pacer
                .set_frame_interval(Some(Duration::from_secs_f64(1.0 / (fps_limit as f64))));
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
        use winit::event::WindowEvent;

        let id = match window::manager::find_id(native_id) {
            window::manager::WindowState::Alive(id) => id,
            window::manager::WindowState::BeingDestroyed => {
                // Window is being destroyed, so we ignore it and just wait for winit to finish cleanup

                if matches!(event, winit::event::WindowEvent::Destroyed) {
                    window::manager::winit_window_destroyed(native_id);
                }

                return;
            }
            window::manager::WindowState::NotFound => {
                // Window not actually found. We made an error with tracking somewhere.
                log::error!(
                    "Could not find WutEngine window for native ID: {}",
                    u64::from(native_id)
                );
                return;
            }
        };

        let was_handled_window_input_event =
            input::insert_raw_window_event(input::WindowIdentifier::from(id), &event);

        if was_handled_window_input_event {
            window::manager::request_redraws();
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                profiling::scope!("Close Requested");

                send_to_main_thread(MainThreadEvent::CloseWindow(id));
            }
            WindowEvent::Resized(_) => {
                profiling::scope!("Resized");

                window::manager::refresh_window(&id, false);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                profiling::scope!("Scale factor changed");

                log::debug!("Window {id} changed scale factor to: {scale_factor}");

                window::manager::refresh_window(&id, false);
            }
            WindowEvent::Focused(_) => {
                profiling::scope!("Focused");

                window::manager::refresh_window(&id, false);
            }
            WindowEvent::Occluded(occluded) => {
                profiling::scope!("Occluded");

                window::manager::notify_window_occluded(&id, occluded);
            }
            WindowEvent::RedrawRequested => {
                self.run_frame();

                if let FrameFrequency::WaitAtMost(secs) = self.frame_frequency {
                    event_loop.set_control_flow(winit::event_loop::ControlFlow::wait_duration(
                        Duration::from_secs_f32(secs),
                    ));
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
        match cause {
            winit::event::StartCause::ResumeTimeReached { .. } => {
                log::debug!("Reached resume time");
                window::manager::request_redraws();
            }
            winit::event::StartCause::WaitCancelled {
                requested_resume: Some(requested_resume),
                ..
            } => {
                event_loop
                    .set_control_flow(winit::event_loop::ControlFlow::WaitUntil(requested_resume));
            }
            _ => {}
        }
    }

    fn user_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        event: MainThreadEvent,
    ) {
        match event {
            MainThreadEvent::NewWindowRequested(window_id, window_config) => {
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
            MainThreadEvent::CloseWindow(window_id) => {
                profiling::scope!("Close Window");

                log::debug!("Handling close window request for window {window_id}");

                let remaining_windows = window::manager::close_window(window_id);

                if remaining_windows == 0 {
                    log::info!(
                        "Stopping the WutEngine runtime because there are no more windows remaining"
                    );
                    event_loop.exit();
                }
            }

            MainThreadEvent::UpdateWindow(window_id, update_event) => {
                window::manager::handle_update(window_id, update_event);
                window::manager::refresh_window(&window_id, false);
            }
            MainThreadEvent::RuntimeExitRequested => {
                log::debug!("Runtime exit was requested. Stopping");
                event_loop.exit();
            }
            MainThreadEvent::ForceSurfaceReconfigure(window_id) => {
                window::manager::refresh_window(&window_id, true);
            }
            MainThreadEvent::AddSystem(manifest) => {
                self.systems.queue_system(manifest);
            }
            MainThreadEvent::Redraw => {
                window::manager::request_redraws();
            }
            MainThreadEvent::RunTask(task) => {
                self.async_pool.insert_task(task);
            }
        }
    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        profiling::function_scope!();

        let _ = event_loop;

        input::insert_raw_device_event(device_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;

        if matches!(self.frame_frequency, FrameFrequency::Fast) {
            window::manager::request_redraws();
        }
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
