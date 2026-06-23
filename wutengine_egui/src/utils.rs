//! Misc. utilities and mapping functions

use wutengine_asset::assets::sampler::FilterMode;
use wutengine_asset::assets::sampler::SerializedSampler;
use wutengine_asset::assets::sampler::WrapMode;
use wutengine_asset::assets::sampler::WrapModeType;
use wutengine_asset::assets::texture::TextureConfig;
use wutengine_asset::assets::texture::TextureFormat;

/// Converts a WutEngine [Vec2](wutengine_math::Vec2) to an [egui::Vec2]
#[inline]
pub const fn to_egui_vec2(v: wutengine_math::Vec2) -> egui::Vec2 {
    egui::Vec2 { x: v.x, y: v.y }
}

/// Converts a WutEngine [Vec2](wutengine_math::Vec2) to an [egui::Pos2]
#[inline]
pub const fn to_egui_pos2(v: wutengine_math::Vec2, scale_factor: f32) -> egui::Pos2 {
    egui::Pos2 {
        x: v.x / scale_factor,
        y: v.y / scale_factor,
    }
}

/// Obtains the texture configuration required for a given [egui ImageData](egui::epaint::ImageData)
#[inline]
pub fn tex_config_from_egui_data(delta: &egui::epaint::ImageData) -> TextureConfig {
    match delta {
        egui::ImageData::Color(_) => TextureConfig {
            width: delta.width() as u32,
            height: delta.height() as u32,
            format: TextureFormat::Rgba8,
        },
    }
}

/// Returns the raw image bytes for a given [egui Image](egui::epaint::ImageData)
#[inline]
pub fn egui_image_bytes(image: &egui::epaint::ImageData) -> &[u8] {
    match image {
        egui::ImageData::Color(color_image) => bytemuck::cast_slice(&color_image.pixels),
    }
}

/// Returns the sampler config for the [egui TextureOptions](egui::TextureOptions)
#[inline]
pub fn sampler_from_egui(options: &egui::TextureOptions) -> SerializedSampler {
    let filtering = filter_mode_from_egui(options.magnification);

    if options.magnification != options.minification {
        log::warn!(
            "Different min/mag filters ({:?}/{:?}) for egui texture. This is not yet supported",
            options.minification,
            options.magnification
        );
    }

    let wrapping = wrap_mode_from_egui(options.wrap_mode);

    SerializedSampler {
        texture_filtering: filtering,
        mipmap_filtering: options
            .mipmap_mode
            .map(filter_mode_from_egui)
            .unwrap_or_default(),
        wrapping,
    }
}

/// Converts an [egui::TextureFilter] to a WutEngine [FilterMode]
#[inline]
pub fn filter_mode_from_egui(egui_mode: egui::TextureFilter) -> FilterMode {
    match egui_mode {
        egui::TextureFilter::Nearest => FilterMode::Nearest,
        egui::TextureFilter::Linear => FilterMode::Linear,
    }
}

/// Converts an [egui::TextureWrapMode] to a WutEngine [WrapModeType]
#[inline]
pub fn wrap_mode_from_egui(egui_mode: egui::TextureWrapMode) -> WrapModeType {
    match egui_mode {
        egui::TextureWrapMode::ClampToEdge => WrapModeType::Single(WrapMode::Clamp),
        egui::TextureWrapMode::Repeat => WrapModeType::Single(WrapMode::Repeat),
        egui::TextureWrapMode::MirroredRepeat => WrapModeType::Single(WrapMode::MirrorRepeat),
    }
}

/// A Rect in physical pixel space, used for setting clipping rectangles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScissorRect {
    /// Scissor X
    pub x: u32,

    /// Scissor Y
    pub y: u32,

    /// Scissor width
    pub width: u32,

    /// Scissor height
    pub height: u32,
}

impl ScissorRect {
    /// Creates a new scissor rect from an egui rect, and a surface configuration
    /// The resulting rect can be used directly in graphics APIs
    pub fn new(
        clip_rect: &egui::epaint::Rect,
        pixels_per_point: f32,
        target_size: (u32, u32),
    ) -> Self {
        // Transform clip rect to physical pixels:
        let clip_min_x = pixels_per_point * clip_rect.min.x;
        let clip_min_y = pixels_per_point * clip_rect.min.y;
        let clip_max_x = pixels_per_point * clip_rect.max.x;
        let clip_max_y = pixels_per_point * clip_rect.max.y;

        // Round to integer:
        let clip_min_x = clip_min_x.round() as u32;
        let clip_min_y = clip_min_y.round() as u32;
        let clip_max_x = clip_max_x.round() as u32;
        let clip_max_y = clip_max_y.round() as u32;

        // Clamp:
        let clip_min_x = clip_min_x.clamp(0, target_size.0);
        let clip_min_y = clip_min_y.clamp(0, target_size.1);
        let clip_max_x = clip_max_x.clamp(clip_min_x, target_size.0);
        let clip_max_y = clip_max_y.clamp(clip_min_y, target_size.0);

        Self {
            x: clip_min_x,
            y: clip_min_y,
            width: clip_max_x - clip_min_x,
            height: clip_max_y - clip_min_y,
        }
    }
}
