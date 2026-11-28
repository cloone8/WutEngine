use core::assert_matches;
use core::cell::Cell;
use core::hint;
use core::time::Duration;
use std::rc::Rc;

use wgpu::Adapter;
use wgpu::Features;
use winit::event_loop::ActiveEventLoop;
use wutengine_renderer as renderer;
use wutengine_renderer::surfaces;
use wutengine_sync::thread_context;

use crate::log;
use crate::runtime;
use crate::runtime::driver::winit::AppPhase;
use crate::runtime::driver::winit::WutEngineApp;
use crate::runtime::frame::FrameState;
use crate::runtime::hooks;
use crate::window::Window;
use crate::window::WindowSettings;
use crate::window::WindowState;

/// Begins the startup of a [`WutEngineApp`] driver, transitioning it to the
/// [Starting](AppPhase::Starting) phase. Must only be called on [non-started drivers](AppPhase::NotStarted).
pub(super) fn driver_startup(driver: &mut WutEngineApp, event_loop: &dyn ActiveEventLoop) {
    profiling::function_scope!();
    thread_context::assert_main_context();

    assert_matches!(
        driver.phase,
        AppPhase::NotStarted,
        "Driver already starting or startup"
    );

    assert!(
        driver.surfaces_allowed,
        "Not allowed to create surfaces, so cannot create the main surface"
    );

    log::info!("Beginning windowing application main startup");

    // First we need a WGPU instance so we can start initializing anything that depends
    // on a renderer
    renderer::init_instance();

    // Next we create the main window and its surface, so we can find a compatible
    // adapter for it
    let main_window = Window::create(WindowSettings::default());

    main_window.mark_main();

    // The previous statements just sent the request. Initialize as a one-off here
    // to make sure there were no errors and we can find the surface
    driver.initialize_pending_windows(event_loop);

    if let WindowState::Error(e) = main_window.state() {
        runtime::fatal_error(format!("Failed to create main window: {e}").as_str());
    }

    let main_surface = driver
        .native_windows
        .get(&main_window)
        .map(|win| win.surface.expect("Missing main surface"))
        .expect("Missing main window");

    log::debug!("Requesting async resources");

    // We set up the adapter using the main surface. We do this
    // now, because there's no guarantee that the surface won't be destroyed
    // if we do it async later. Note that only the request uses the surface. We
    // can await the future freely, because the surface is no longer needed there
    let init_adapter_fut = surfaces::get(main_surface, |main_surface| {
        renderer::init_adapter(main_surface)
    });

    // Now we created the task that gets the adapter, we can safely
    // defer the rest of the initialization to later
    let ready_signal = Rc::new(Cell::new(false));

    driver.main_thread_executor.spawn(initialize_renderer(
        init_adapter_fut,
        Rc::clone(&ready_signal),
    ));

    // Now we wait for startup to be done
    driver.phase = AppPhase::Starting {
        ready: ready_signal,
    };
}

/// For drivers in the [startup phase](AppPhase::Starting), checks if startup is done
/// and transitions to the [running phase](AppPhase::Running) if so.
///
/// For other phases, returns immediately, and is thus safe to call
#[inline]
pub(super) fn finish_initialization(driver: &mut WutEngineApp) {
    let AppPhase::Starting { ready } = &driver.phase else {
        // Common case: Driver already running
        return;
    };

    hint::cold_path();

    if !ready.get() {
        return;
    }

    log::debug!("Async startup ready signal given.");

    hooks::run_started_hooks();

    driver.phase = AppPhase::Running {
        frame_state: FrameState::new(Duration::from_secs_f64(1.0 / 60.0)), // TODO:
                                                                           // Make
                                                                           // configurable?
    };

    profiling::finish_frame!(); // So we can track the entire startup phase
}

async fn initialize_renderer(
    init_adapter_fut: impl Future<Output = Option<(Adapter, Features)>>,
    ready_signal: Rc<Cell<bool>>,
) {
    // Wait for WGPU to set up the adapter
    let Some((adapter, adapter_features)) = init_adapter_fut.await else {
        runtime::fatal_error("Failed to set up adapter");
    };

    // And with the adapter and its features, we can set up the actual device and the rest of the
    // renderer
    if !renderer::init_renderer(adapter, adapter_features).await {
        runtime::fatal_error("Failed to set up renderer");
    }

    log::info!("Renderer device initialization done");

    ready_signal.set(true);
}
