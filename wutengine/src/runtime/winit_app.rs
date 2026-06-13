//! Implements the [winit::application::ApplicationHandler] interface for [crate::runtime::Runtime],
//! so that its execution can be driven by [winit]

use core::time::Duration;

use alloc::sync::Arc;

use crate::config;
use crate::input;
use crate::runtime::notify_event_loop;
use crate::window::{Window, WindowConfig};
use crate::{graphics, thread, time, window};

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

    /// Surface should be reconfigured
    ForceSurfaceReconfigure(Window),

    /// Someone requested the exit of the runtime through [crate::runtime::exit]
    RuntimeExitRequested,
}

impl winit::application::ApplicationHandler<WinitEvent> for Runtime {
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
        thread::init_thread_pool();

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

        profiling::function_scope!();

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
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                profiling::scope!("Close Requested");

                notify_event_loop(WinitEvent::CloseWindow(id));
            }
            WindowEvent::Resized(_) => {
                profiling::scope!("Resized");

                window::manager::refresh_window(&id, false);

                if cfg!(windows) {
                    // Workaround for https://github.com/rust-windowing/winit/issues/3272
                    // The frame still freezes, but at least the whole window is redrawn once
                    // to make sure it's filled
                    self.about_to_wait(event_loop);
                }
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
                window::manager::request_redraw(id);
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

                let remaining_windows = window::manager::close_window(window_id);

                if remaining_windows == 0 {
                    log::info!(
                        "Stopping the WutEngine runtime because there are no more windows remaining"
                    );
                    event_loop.exit();
                }
            }
            WinitEvent::UpdateIcon(window_id, icon) => {
                profiling::scope!("Update Icon");

                log::debug!("Handling icon update request for window {window_id}");

                window::manager::set_icon(window_id, icon);
            }
            WinitEvent::RuntimeExitRequested => {
                log::debug!("Runtime exit was requested. Stopping");
                event_loop.exit();
            }
            WinitEvent::ForceSurfaceReconfigure(window_id) => {
                window::manager::refresh_window(&window_id, true);
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

        // Toggle profiling scopes
        crate::profiling::change_scope_active_status();

        // We wait for the rendering target to become available in the beginning of the frame,
        // because then if we block on vsync or similar the simulation will not be out of date
        let surfaces = window::manager::get_surface_textures();

        input::gamepad::poll_for_events();

        let dev_overlay_output = Self::prepare_development_overlay(&surfaces);

        self.run_frame_logic();

        self.render_all_windows(&surfaces);

        if let Some((dev_overlay_window, dev_overlay_ready_chan)) = dev_overlay_output {
            #[cfg(not(feature = "development_overlay"))]
            {
                _ = (dev_overlay_window, dev_overlay_ready_chan);

                unsafe {
                    wutengine_util::unreachable_dbg!();
                }
            }

            #[cfg(feature = "development_overlay")]
            {
                {
                    profiling::scope!("Wait for dev overlay");
                    // Wait for the development overlay to be ready, and actually render it to the surface
                    dev_overlay_ready_chan.recv().unwrap();
                }

                let (_, dev_overlay_surface) = surfaces
                    .iter()
                    .find(|(win, _)| *win == dev_overlay_window)
                    .unwrap();

                if let Some(dev_overlay_command_buf) =
                    crate::development_overlay::render_overlay(dev_overlay_surface)
                {
                    graphics::queue().submit([dev_overlay_command_buf]);
                }
            }
        }

        for (_, surface) in surfaces {
            surface.present();
        }

        input::end_frame();

        self.frame_pacer.frame_rendered();
        self.frame_pacer.wait_for_limit();

        profiling::finish_frame!();
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
