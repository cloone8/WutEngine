use std::sync::Arc;

use wgpu::Instance;
use wgpu::Surface;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;
use winit::window::WindowAttributes;

use crate::log;
use crate::renderer;

pub(super) fn startup_sync(
    active_el: &dyn ActiveEventLoop,
) -> (Instance, Arc<dyn Window>, Surface<'static>) {
    log::debug!("Beginning start-up");

    // Now we need an instance and a surface in order to find a compatible adapter
    let instance = renderer::init_instance();

    let main_window = active_el
        .create_window(WindowAttributes::default())
        .expect("Failed to create main window");

    let main_window_shared: Arc<dyn Window> = Arc::from(main_window);

    let main_surface = instance
        .create_surface(Arc::clone(&main_window_shared))
        .expect("Failed to create surface");

    (instance, main_window_shared, main_surface)
}

/// Main startup function that sets up the full engine context and runs the app entrypoint
pub(super) async fn startup_async(instance: Instance, main_surface: &Surface<'_>) {
    log::debug!("Requesting async resources");

    // With the instance and surface, we can fully set up the rest of
    // the renderer
    if !renderer::init_renderer(instance, main_surface).await {
        //TODO: Better handling
        panic!("Failed to set up renderer");
    }

    log::info!("Start-up done");
}
