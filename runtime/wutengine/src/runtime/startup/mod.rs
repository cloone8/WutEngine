use crate::log;
use crate::renderer;
use crate::window;
use crate::window::Window;

pub(super) fn startup_sync() -> Window {
    log::debug!("Beginning start-up");

    // Start the window manager
    window::init();

    // Now we need an instance and a surface in order to find a compatible adapter
    renderer::init_instance();

    let main_window = window::create();

    main_window
}

/// Main startup function that sets up the full engine context and runs the app entrypoint
pub(super) async fn startup_async(main_window: Window) {
    log::debug!("Requesting async resources");

    let init_renderer_fut = renderer::get_surface(main_window, |main_surface| {
        renderer::init_renderer(main_surface)
    });

    // With the instance and surface, we can fully set up the rest of
    // the renderer
    if !init_renderer_fut.await {
        //TODO: Better handling
        panic!("Failed to set up renderer");
    }

    log::info!("Start-up done");
}
