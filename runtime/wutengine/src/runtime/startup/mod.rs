use core::panic;
use core::time::Duration;

use wutengine_renderer as renderer;
use wutengine_renderer::surfaces;
use wutengine_renderer::surfaces::SurfaceId;

use crate::log;
use crate::time;
use crate::window;
use crate::window::Window;
use crate::window::WindowSettings;

pub(super) fn startup_sync() -> Window {
    log::debug!("Beginning start-up");

    // Start the window manager
    window::init();

    // Now we need an instance and a surface in order to find a compatible adapter
    renderer::init_instance();

    time::set_fixed_delta(Duration::from_secs_f64(1.0 / 60.0).as_nanos() as u64); // TODO: Make configurable

    Window::create(WindowSettings::default())
}

/// Main startup function that sets up the full engine context and runs the app entrypoint
pub(super) async fn startup_async(main_surface_id: SurfaceId) {
    log::debug!("Requesting async resources");

    // First we set up the adapter using the main surface.
    let init_adapter = surfaces::get(main_surface_id, |main_surface| {
        renderer::init_adapter(main_surface)
    });

    // Wait for WGPU to set up the adapter
    let Some((adapter, adapter_features)) = init_adapter.await else {
        panic!("Failed to set up adapter");
    };

    // And with the adapter and its features, we can set up the actual device and the rest of the
    // renderer
    if !renderer::init_renderer(adapter, adapter_features).await {
        panic!("Failed to set up renderer");
    }

    log::info!("Start-up done");
}
