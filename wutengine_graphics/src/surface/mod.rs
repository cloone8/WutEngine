use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

use thiserror::Error;
use wutengine_util::GlobalManager;
use wutengine_windowing::window::{WindowIdentifier, WindowResizedEvent};

use crate::GRAPHICS_MANAGER;

static SURFACE_MANAGER: GlobalManager<SurfaceManager> = GlobalManager::new();

pub(crate) fn init() {
    GlobalManager::init(&SURFACE_MANAGER, SurfaceManager::new());
}

#[derive(Debug)]
struct SurfaceManager {
    surfaces: RwLock<HashMap<WindowIdentifier, Surface>>,
    frame_surfaces: Mutex<Vec<wgpu::SurfaceTexture>>,
    frame_surface_views: RwLock<HashMap<WindowIdentifier, wgpu::TextureView>>,
}

impl SurfaceManager {
    fn new() -> Self {
        Self {
            surfaces: RwLock::new(HashMap::default()),
            frame_surfaces: Mutex::new(Vec::new()),
            frame_surface_views: RwLock::new(HashMap::default()),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Surface {
    native: wgpu::Surface<'static>,
    capabilities: wgpu::SurfaceCapabilities,
}

impl Surface {
    fn reconfigure(&self, device: &wgpu::Device, size: (u32, u32)) {
        let surface_format = self.capabilities.formats[0];

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: size.0,
            height: size.1,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };

        self.native.configure(device, &surface_config);
    }
}

/// An error initializing a surface with [initialize_for_window]
#[derive(Debug, Error)]
pub enum InitSurfaceErr {
    /// Could not craete the rendering surface due to a native error
    #[error("Failed to create a rendering surface for the given window: {0}")]
    CreateSurface(#[from] wgpu::CreateSurfaceError),

    /// The current graphics adapter does not support rendering to the provided surface
    #[error("The current adapter does not support rendering to the window")]
    UnsupportedAdapter,
}

/// Initializes a new graphics surface for the given window identifier and native surface target.
/// The initial surface size is given by `inner_size`
pub fn initialize_for_window(
    id: WindowIdentifier,
    inner_size: (u32, u32),
    native: impl Into<wgpu::SurfaceTarget<'static>>,
) -> Result<(), InitSurfaceErr> {
    let surface = GRAPHICS_MANAGER.instance.create_surface(native)?;

    if !GRAPHICS_MANAGER.adapter.is_surface_supported(&surface) {
        return Err(InitSurfaceErr::UnsupportedAdapter);
    }

    let graphics_surface = Surface {
        capabilities: surface.get_capabilities(&GRAPHICS_MANAGER.adapter),
        native: surface,
    };

    graphics_surface.reconfigure(&GRAPHICS_MANAGER.device, inner_size);

    let mut locked = SURFACE_MANAGER.surfaces.write().unwrap();
    locked.insert(id, graphics_surface);

    Ok(())
}

pub(crate) fn on_resized_event(event: &WindowResizedEvent) {
    let id = &event.window_id;
    let inner_size = event.new_size;

    let mut surfaces = SURFACE_MANAGER.surfaces.write().unwrap();

    let surface = if let Some(surface) = surfaces.get_mut(id) {
        log::debug!("Resizing window surface {id}");
        surface
    } else {
        log::warn!("Resizing unknown surface {id}. Ignoring");
        return;
    };

    surface.reconfigure(&GRAPHICS_MANAGER.device, inner_size);
}

/// For all known surfaces, retrieve the surface texture and cache their views
#[profiling::function]
pub fn get_all_surface_textures() {
    log::trace!("Getting all surface textures");

    let surfaces = SURFACE_MANAGER.surfaces.read().unwrap();
    let mut frame_surfaces = SURFACE_MANAGER.frame_surfaces.lock().unwrap();
    let mut frame_surface_views = SURFACE_MANAGER.frame_surface_views.write().unwrap();

    let default_desc = wgpu::TextureViewDescriptor::default();

    for (id, surface) in surfaces.iter() {
        let surface_texture = surface
            .native
            .get_current_texture()
            .expect("Failed to acquire next swapchain texture");

        let surface_texture_view = surface_texture.texture.create_view(&default_desc);

        frame_surfaces.push(surface_texture);
        frame_surface_views.insert(id.clone(), surface_texture_view);
    }
}

/// Presents all surfaces for which at least one view was requested (with [get_surface_texture_view]) this frame
#[profiling::function]
pub fn present_all() {
    log::trace!("Presenting surfaces");

    SURFACE_MANAGER.frame_surface_views.write().unwrap().clear();

    let mut frame_sufraces = SURFACE_MANAGER.frame_surfaces.lock().unwrap();

    let mut i = 0;

    for to_present in frame_sufraces.drain(..) {
        to_present.present();
        i += 1;
    }

    log::trace!("Presented {i} surfaces");
}

/// Returns the cached surface texture view for the given surface, for the current frame
#[profiling::function]
pub fn get_surface_texture_view(id: &WindowIdentifier) -> Option<wgpu::TextureView> {
    let surface_views_read = SURFACE_MANAGER.frame_surface_views.read().unwrap();

    surface_views_read.get(id).cloned()
}
