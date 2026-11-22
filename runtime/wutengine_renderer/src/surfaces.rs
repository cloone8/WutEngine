//! Rendering surface management

use std::collections::HashMap;
use std::sync::OnceLock;

use wgpu::CompositeAlphaMode;
use wgpu::CurrentSurfaceTexture;
use wgpu::DownlevelFlags;
use wgpu::PresentMode;
use wgpu::Surface;
use wgpu::SurfaceColorSpace;
use wgpu::SurfaceColorSpaces;
use wgpu::SurfaceConfiguration;
use wgpu::SurfaceFormatCapabilities;
use wgpu::SurfaceTarget;
use wgpu::SurfaceTexture;
use wgpu::TextureFormat;
use wgpu::TextureUsages;
use wgpu::TextureView;
use wgpu::wgt::TextureViewDescriptor;
use wutengine_macro::unique_id_type32;
use wutengine_sync::RwLock;

use crate::adapter;
use crate::device;
use crate::instance;
use crate::log;
use crate::queue;

unique_id_type32! {
    /// Identifier for a single surface
    pub SurfaceId
}

static SURFACES: OnceLock<RwLock<HashMap<SurfaceId, SurfaceData>>> = OnceLock::new();

type PrePresentFn = dyn Fn() + Send + Sync;

#[derive(derive_more::Debug)]
struct SurfaceData {
    native_surface: Surface<'static>,
    configured: bool,
    surface_type: SurfaceType,
    size: (u32, u32),

    #[debug(skip)]
    pre_present_callback: Option<Box<PrePresentFn>>,
}

impl SurfaceData {
    fn new(
        native_surface: Surface<'static>,
        surface_type: SurfaceType,
        size: (u32, u32),
        pre_present_callback: Option<Box<PrePresentFn>>,
    ) -> Self {
        Self {
            native_surface,
            surface_type,
            size,
            pre_present_callback,
            configured: false,
        }
    }
}

fn surfaces() -> &'static RwLock<HashMap<SurfaceId, SurfaceData>> {
    SURFACES.get().expect("Surfaces not yet initialized")
}

/// Initializes the surfaces map. Should just be called once
pub(super) fn init_surface_map() {
    SURFACES
        .set(RwLock::new(HashMap::new()))
        .expect("Surface map already initialized");
}

/// Initializes the rendering surface for the given native `surface_id`, with the given handle
pub fn initialize(
    surface_type: SurfaceType,
    surface_size: (u32, u32),
    native: impl Into<SurfaceTarget<'static>>,
    pre_present_callback: Option<Box<PrePresentFn>>,
) -> SurfaceId {
    profiling::function_scope!();

    let surface_id = SurfaceId::new();

    let surface = instance()
        .create_surface(native)
        .expect("Failed to initialize surface for surface_id"); //TODO: Handle properly

    surfaces().write().insert(
        surface_id,
        SurfaceData::new(surface, surface_type, surface_size, pre_present_callback),
    );

    surface_id
}

/// Removes the rendering surface for the given `surface_id`
pub fn remove(surface_id: SurfaceId) {
    profiling::function_scope!();

    let surface = surfaces().write().remove(&surface_id);

    if surface.is_none() {
        log::error!("Failed to remove surface for unknown surface_id {surface_id}");
    }
}

/// Runs the callback with the [`Surface`] for the given `surface_id`
pub fn get<T>(surface_id: SurfaceId, cb: impl FnOnce(&Surface<'static>) -> T) -> T {
    profiling::function_scope!();

    let surfaces = surfaces().read();

    let surface = surfaces
        .get(&surface_id)
        .expect("Missing surface for surface_id");

    cb(&surface.native_surface)
}

/// An acquired rendering surface
#[derive(Debug)]
pub(crate) struct AcquiredSurface {
    /// The ID of the surface
    pub(crate) id: SurfaceId,

    /// The actual surface texture
    pub(crate) texture: SurfaceTexture,

    /// The view into the surface texture
    pub(crate) view: TextureView,
}

impl AcquiredSurface {
    /// Presents this surface texture
    pub(crate) fn present(self) {
        profiling::function_scope!();

        let surfaces = surfaces().read();

        let surface_data = surfaces.get(&self.id).expect("Missing surface");

        if let Some(pre_present) = surface_data.pre_present_callback.as_ref() {
            pre_present();
        }

        queue().present(self.texture);
    }
}

/// Attempts to acquire all known surfaces. If any surfaces are not yet configured,
/// this function does so. Returns an [`AcquiredSurface`] for each surface that was successfully
/// acquired. Note that this does not mean that _all_ surfaces were acquired
pub(crate) fn acquire_surfaces() -> Vec<AcquiredSurface> {
    profiling::function_scope!();

    let mut surfaces = surfaces().write();

    let mut output_surfaces = Vec::with_capacity(surfaces.len());

    for (surface, data) in surfaces.iter_mut() {
        if data.size.0 == 0 || data.size.1 == 0 {
            // Zero sized surfaces are not valid render targets
            continue;
        }

        if !data.configured {
            configure(*surface, data);
        }

        let Some(surface_texture) = acquire_surface_texture_for_surface(*surface, data) else {
            // If we're unable to acquire the surface texture for whatever reason, we'll skip rendering
            // to this window
            continue;
        };

        output_surfaces.push(AcquiredSurface {
            id: *surface,
            view: surface_texture
                .texture
                .create_view(&TextureViewDescriptor::default()),
            texture: surface_texture,
        });
    }

    output_surfaces
}

fn acquire_surface_texture_for_surface(
    surface_id: SurfaceId,
    surface_data: &mut SurfaceData,
) -> Option<SurfaceTexture> {
    profiling::function_scope!();

    match surface_data.native_surface.get_current_texture() {
        CurrentSurfaceTexture::Success(surface_texture) => Some(surface_texture),
        CurrentSurfaceTexture::Suboptimal(surface_texture) => {
            // Make sure we reconfigure the next time we acquire surfaces
            surface_data.configured = false;
            Some(surface_texture)
        }
        CurrentSurfaceTexture::Timeout => {
            log::warn!("Skipping rendering to surface {surface_id} due to timeout");
            None
        }
        CurrentSurfaceTexture::Occluded => None,
        CurrentSurfaceTexture::Outdated => {
            surface_data.configured = false;
            None
        }
        CurrentSurfaceTexture::Lost => {
            surface_data.configured = false;
            None
        }
        CurrentSurfaceTexture::Validation => {
            log::error!("Validation error when acquiring surface {surface_id}");
            None
        }
    }
}

/// Updates the size of the given surface. The size is the physical size (in pixels)
pub fn change_surface_size(surface_id: SurfaceId, size: (u32, u32)) {
    profiling::function_scope!();
    log::trace!(
        "Updating size for surface {surface_id} to {}x{}",
        size.0,
        size.1
    );

    let mut surfaces = surfaces().write();

    let Some(surface) = surfaces.get_mut(&surface_id) else {
        log::error!("Cannot find surface for surface_id {surface_id}");
        return;
    };

    surface.size = size;
    surface.configured = false;
}

/// Reconfigures the surface for the given `surface_id`
fn configure(surface_id: SurfaceId, surface: &mut SurfaceData) {
    profiling::function_scope!();

    let capabilities = surface.native_surface.get_capabilities(adapter());

    let (format, color_space) = get_best_format(&capabilities.format_capabilities, HdrMode::Sdr);
    let present_mode = get_best_present_mode(
        &capabilities.present_modes,
        surface.surface_type,
        VsyncSetting::On,
    );

    // Only an sRGB surface needs an extra view format: its non-sRGB variant, if the adapter allows surface view
    // formats. Otherwise the view uses the configured format itself
    let view_formats = if format.is_srgb()
        && adapter()
            .get_downlevel_capabilities()
            .flags
            .contains(DownlevelFlags::SURFACE_VIEW_FORMATS)
    {
        vec![format.remove_srgb_suffix()]
    } else {
        Vec::new()
    };

    let surface_configuration = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format,
        color_space,
        width: surface.size.0,
        height: surface.size.1,
        present_mode,
        desired_maximum_frame_latency: 2, // TODO: Make configurable
        alpha_mode: CompositeAlphaMode::Opaque,
        view_formats,
    };

    log::debug!("Configuring surface for surface_id {surface_id} to: {surface_configuration:#?}");

    surface
        .native_surface
        .configure(device(), &surface_configuration);

    surface.configured = true;
}

/// Placeholder for later when we might support HDR
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HdrMode {
    /// Standard dynamic range
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

/// Returns the most preferred format with the sRGB color space, preferring formats without an sRGB suffix since
/// output encoding happens in the renderer. Only falls back to an sRGB format if the surface offers nothing else
fn get_best_sdr_format(supported_formats: &[SurfaceFormatCapabilities]) -> Option<TextureFormat> {
    // First we find all non-HDR color spaces
    let srgb_color_space_formats = supported_formats
        .iter()
        .filter(|format_capability| {
            format_capability
                .color_spaces
                .contains(SurfaceColorSpaces::SRGB)
        })
        .map(|format_capability| format_capability.format)
        .collect::<Vec<_>>();

    // Then, for those color spaces, we try to find one that has a non-sRGB-encoding
    // texture format
    for color_space_format in &srgb_color_space_formats {
        if !color_space_format.is_srgb() {
            return Some(*color_space_format);
        }
    }

    // If we can't find one, we simply pick the first supported one
    srgb_color_space_formats.first().copied()
}

/// The vsync setting for the main surface
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VsyncSetting {
    /// Vsync on: [`PresentMode::Fifo`]
    On,

    /// [`PresentMode::Mailbox`] if supported, [`PresentMode::Fifo`] otherwise
    Fast,
    /// [`PresentMode::FifoRelaxed`] if supported, [`PresentMode::Fifo`] otherwise
    Adaptive,

    /// First supported out of [`PresentMode::Immediate`], [`PresentMode::Mailbox`] and
    /// [`PresentMode::Fifo`]
    Off,
}

/// The type of the surface. The main surface is used for pacing the entire game,
/// so has more options to control it. Other `surface_ids` always choose the best supported
/// mode that follows the main surface
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceType {
    /// The main surface
    Main,

    /// Any other surface
    Other,
}

fn get_best_present_mode(
    supported_modes: &[PresentMode],
    surface_type: SurfaceType,
    vsync: VsyncSetting,
) -> PresentMode {
    match surface_type {
        SurfaceType::Main => match vsync {
            VsyncSetting::On => PresentMode::Fifo,
            VsyncSetting::Fast => first_supported_out_of(supported_modes, &[PresentMode::Mailbox]),
            VsyncSetting::Adaptive => {
                first_supported_out_of(supported_modes, &[PresentMode::FifoRelaxed])
            }
            VsyncSetting::Off => first_supported_out_of(
                supported_modes,
                &[PresentMode::Immediate, PresentMode::Mailbox],
            ),
        },
        SurfaceType::Other => first_supported_out_of(
            supported_modes,
            &[PresentMode::Mailbox, PresentMode::Immediate],
        ),
    }
}

fn first_supported_out_of(supported_modes: &[PresentMode], to_try: &[PresentMode]) -> PresentMode {
    for present_mode in to_try {
        if supported_modes.contains(present_mode) {
            return *present_mode;
        }
    }

    // Fallback is always FIFO, since that's the only mode always supported everywhere
    PresentMode::Fifo
}
