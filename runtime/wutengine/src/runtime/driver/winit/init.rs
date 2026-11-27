use core::panic;

use wutengine_renderer as renderer;
use wutengine_renderer::surfaces;
use wutengine_renderer::surfaces::SurfaceId;
use wutengine_sync::thread_context;

use crate::log;
use crate::window::Window;
use crate::window::WindowSettings;

pub(super) fn initialize_renderer_and_main_window() -> Window {
    thread_context::assert_main_context();

    log::debug!("Initializing renderer instance");

    // Now we need an instance and a surface in order to find a compatible adapter
    renderer::init_instance();

    let window = Window::create(WindowSettings::default());

    window.mark_main();

    window
}
pub(super) async fn initialize_main_surface_and_device(main_surface_id: SurfaceId) {
    thread_context::assert_main_context();

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

    log::info!("Renderer device initialization done");
}
