//! Rendering surface management

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;

use wgpu::CompositeAlphaMode;
use wgpu::PresentMode;
use wgpu::Surface;
use wgpu::SurfaceColorSpace;
use wgpu::SurfaceColorSpaces;
use wgpu::SurfaceConfiguration;
use wgpu::SurfaceFormatCapabilities;
use wgpu::TextureFormat;
use wgpu::TextureUsages;
use wutengine_sync::RwLock;

use crate::log;
use crate::renderer;
use crate::window::Window;

use winit::window::Window as WinitWindow;

static SURFACES: OnceLock<RwLock<HashMap<Window, SurfaceData>>> = OnceLock::new();

#[derive(Debug)]
struct SurfaceData {
    surface: Surface<'static>,
    configured: bool,
}

impl SurfaceData {
    fn new(surface: Surface<'static>) -> Self {
        Self {
            surface,
            configured: false,
        }
    }
}

fn surfaces() -> &'static RwLock<HashMap<Window, SurfaceData>> {
    SURFACES.get().expect("Surfaces not yet initialized")
}

pub(super) fn init_surface_map() {
    SURFACES
        .set(RwLock::new(HashMap::new()))
        .expect("Surface map already initialized");
}

/// Initializes the rendering surface for the given native window, with the given handle
pub(crate) fn initialize(window: Window, native: Arc<dyn WinitWindow>) {
    profiling::function_scope!();

    let surface = renderer::instance()
        .create_surface(native)
        .expect("Failed to initialize surface for window"); //TODO: Handle properly

    surfaces().write().insert(window, SurfaceData::new(surface));
}

/// Runs the callback with the [`Surface`] for the given window
pub(crate) fn get<T>(window: Window, cb: impl FnOnce(&Surface<'static>) -> T) -> T {
    profiling::function_scope!();

    let surfaces = surfaces().read();

    let surface = surfaces.get(&window).expect("Missing surface for window");

    cb(&surface.surface)
}

/// Returns whether the surface for the given window is configured
pub(crate) fn is_configured(window: Window) -> bool {
    surfaces()
        .read()
        .get(&window)
        .expect("Unknown window")
        .configured
}

/// Marks the surface for the given window as "unconfigured", causing the runtime to
/// reconfigure it the next time that it's needed
pub(crate) fn mark_unconfigured(window: Window) {
    profiling::function_scope!();
    log::trace!("Marking surface for window {window} as unconfigured");

    let mut surfaces = surfaces().write();

    let Some(surface) = surfaces.get_mut(&window) else {
        log::error!("Cannot find surface for window {window}");
        return;
    };

    surface.configured = false;
}

/// Reconfigures the surface for the given window
pub(crate) fn configure(window: Window, size: (u32, u32)) {
    profiling::function_scope!();

    let mut surfaces = surfaces().write();

    let surface = surfaces
        .get_mut(&window)
        .expect("Missing surface for window");

    let capabilities = surface.surface.get_capabilities(renderer::adapter());

    let (format, color_space) = get_best_format(&capabilities.format_capabilities, HdrMode::Sdr);
    let present_mode = get_best_present_mode(&capabilities.present_modes, false); // TODO: Make this configurable

    let surface_configuration = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format,
        color_space,
        width: size.0,
        height: size.1,
        present_mode,
        desired_maximum_frame_latency: 2, // TODO: Make configurable
        alpha_mode: CompositeAlphaMode::Opaque,
        view_formats: vec![format.remove_srgb_suffix(), format.add_srgb_suffix()],
    };

    log::debug!("Configuring surface for window {window} to: {surface_configuration:#?}");

    surface
        .surface
        .configure(renderer::device(), &surface_configuration);

    surface.configured = true;
}

/// Placeholder for later when we might support HDR
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HdrMode {
    Sdr,
}

/// Given an ordered list of supported formats (most prefered first),
/// returns the first format that matches the given color space preference.
/// If `hdr` is not [`HdrMode::Sdr`], tries to find an HDR format. If that's not possible,
/// returns an SDR format instead.
fn get_best_format(
    supported_formats: &[SurfaceFormatCapabilities],
    hdr: HdrMode,
) -> (TextureFormat, SurfaceColorSpace) {
    assert!(
        !supported_formats.is_empty(),
        "Must be given at least one format"
    );

    if hdr == HdrMode::Sdr {
        return (
            get_best_sdr_format(supported_formats).expect("Should find at least one SDR format"),
            SurfaceColorSpace::Srgb,
        );
    }

    panic!("HDR Output is not yet implemented")
}

fn get_best_sdr_format(supported_formats: &[SurfaceFormatCapabilities]) -> Option<TextureFormat> {
    supported_formats
        .iter()
        .find(|format_capability| {
            format_capability
                .color_spaces
                .contains(SurfaceColorSpaces::SRGB)
        })
        .map(|format_capability| format_capability.format)
}

fn get_best_present_mode(supported_modes: &[PresentMode], vsync: bool) -> PresentMode {
    // TODO: Make a better selection once we allow configuring the present mode
    // with a config file
    _ = supported_modes;

    if vsync {
        PresentMode::AutoVsync
    } else {
        PresentMode::AutoNoVsync
    }
}
