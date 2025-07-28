//! The [winit] application handler abstraction. Responds to [winit] callbacks and events, and calls
//! the right functions in the main WutEngine runtime as a response.

use core::fmt::Debug;
use std::sync::Arc;

use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use wutengine_windowing::display;

use crate::config::InitialWindowConfig;
use crate::runtime::{render, run_step};
use crate::window::{WindowMode, WindowOptions};
use crate::{WutEngineWinitEvent, window};

/// Data needed for the initial setup of the [winit] environment
pub(crate) struct WinitInitData {
    /// The initial window to create, if any
    pub(crate) initial_window: Option<InitialWindowConfig>,

    /// The callback to call once the initial setup of the winit runtieme is done (in [winit::application::ApplicationHandler::resumed])
    pub(crate) post_init_callback: Option<Box<dyn FnOnce()>>,
}

impl Debug for WinitInitData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WinitInitData")
            .field("initial_window", &self.initial_window)
            .field(
                "post_init_callback",
                if self.post_init_callback.is_some() {
                    &"<has callback>"
                } else {
                    &"<no callback>"
                },
            )
            .finish()
    }
}

/// The [winit] application handler. Calls the corresponding WutEngine runtime functions for each winit event
#[derive(Debug)]
pub(crate) struct WinitApp {
    /// If [Some], WutEngine hasn't been configured yet
    init_data: Option<WinitInitData>,

    event_loop: EventLoopProxy<WutEngineWinitEvent>,
}

impl WinitApp {
    /// A new [WinitApp]
    pub(crate) fn new(
        event_loop: EventLoopProxy<WutEngineWinitEvent>,
        config: WinitInitData,
    ) -> Self {
        Self {
            init_data: Some(config),
            event_loop,
        }
    }
}

#[profiling::all_functions]
impl winit::application::ApplicationHandler<WutEngineWinitEvent> for WinitApp {
    #[profiling::skip]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.init_data.is_none() {
            return;
        }

        profiling::scope!("Initialize WutEngine Windowing");

        log::info!("Starting WutEngine event loop");

        // To initialize the windowing system we need the Winit event loop, which
        // is why it happens only here instead of earlier
        wutengine_windowing::init(event_loop, self.event_loop.clone());

        let init_data = self.init_data.take().unwrap();

        if let Some(initial_window_cfg) = init_data.initial_window {
            let mode = match initial_window_cfg.mode {
                crate::config::InitialWindowMode::Windowed => WindowMode::Windowed,
                crate::config::InitialWindowMode::BorderlessFullscreen => {
                    WindowMode::BorderlessFullscreen(Some(display::main_display().id().clone()))
                }
                crate::config::InitialWindowMode::ExclusiveFullscreen => todo!(),
            };

            window::create(
                initial_window_cfg.id,
                WindowOptions {
                    title: initial_window_cfg.title,
                    mode,
                },
            );
        }

        if let Some(post_init) = init_data.post_init_callback {
            post_init();
        }
    }

    #[profiling::skip]
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        run_step();
        render();
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: WutEngineWinitEvent) {
        match event {
            WutEngineWinitEvent::CreateWindowRequested(window_identifier, window_attributes) => {
                log::info!("Creating window with ID {window_identifier}");

                let window = match event_loop.create_window(window_attributes) {
                    Ok(window) => Arc::new(window),
                    Err(e) => {
                        log::error!("Failed to create window with ID {window_identifier}: {e}");
                        return;
                    }
                };

                wutengine_graphics::initialize_surface_for_window(
                    window_identifier.clone(),
                    window.inner_size().into(),
                    window.clone(),
                )
                .expect("Failed to initialize rendering surface for window");

                window::register_window(window_identifier.clone(), window);
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let identifier = wutengine_windowing::window::identifier_for_native_id(&window_id)
            .expect("Could not find identifier for native winit ID");

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                wutengine_event::publish(wutengine_windowing::window::WindowResizedEvent {
                    window_id: identifier,
                    new_size: new_size.into(),
                });

                if cfg!(target_os = "windows") {
                    // hack for resizing bug in winit, remove once fixed
                    self.about_to_wait(event_loop);
                }
            }
            _ => {}
        };
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        log::info!("WutEngine shutting down");

        log::logger().flush();
    }
}
