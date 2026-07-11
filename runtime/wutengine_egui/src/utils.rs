//! Misc. utilities and mapping functions

use wutengine_assets::assets::{
    sampler::{FilterMode, SerializedSampler, WrapMode, WrapModeType},
    texture::{TextureConfig, TextureFormat},
};

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
        let clip_max_y = clip_max_y.clamp(clip_min_y, target_size.1);

        Self {
            x: clip_min_x,
            y: clip_min_y,
            width: clip_max_x - clip_min_x,
            height: clip_max_y - clip_min_y,
        }
    }
}

/// Maps an [egui cursor icon](egui::CursorIcon) to a [WutEngine cursor icon](cursor_icon::CursorIcon)
pub const fn cursor_icon_from_egui(eg: egui::CursorIcon) -> Option<cursor_icon::CursorIcon> {
    Some(match eg {
        egui::CursorIcon::None => return None,
        egui::CursorIcon::Default => cursor_icon::CursorIcon::Default,
        egui::CursorIcon::ContextMenu => cursor_icon::CursorIcon::ContextMenu,
        egui::CursorIcon::Help => cursor_icon::CursorIcon::Help,
        egui::CursorIcon::PointingHand => cursor_icon::CursorIcon::Pointer,
        egui::CursorIcon::Progress => cursor_icon::CursorIcon::Progress,
        egui::CursorIcon::Wait => cursor_icon::CursorIcon::Wait,
        egui::CursorIcon::Cell => cursor_icon::CursorIcon::Cell,
        egui::CursorIcon::Crosshair => cursor_icon::CursorIcon::Crosshair,
        egui::CursorIcon::Text => cursor_icon::CursorIcon::Text,
        egui::CursorIcon::VerticalText => cursor_icon::CursorIcon::VerticalText,
        egui::CursorIcon::Alias => cursor_icon::CursorIcon::Alias,
        egui::CursorIcon::Copy => cursor_icon::CursorIcon::Copy,
        egui::CursorIcon::Move => cursor_icon::CursorIcon::Move,
        egui::CursorIcon::NoDrop => cursor_icon::CursorIcon::NoDrop,
        egui::CursorIcon::NotAllowed => cursor_icon::CursorIcon::NotAllowed,
        egui::CursorIcon::Grab => cursor_icon::CursorIcon::Grab,
        egui::CursorIcon::Grabbing => cursor_icon::CursorIcon::Grabbing,
        egui::CursorIcon::AllScroll => cursor_icon::CursorIcon::AllScroll,
        egui::CursorIcon::ResizeHorizontal => cursor_icon::CursorIcon::ColResize,
        egui::CursorIcon::ResizeNeSw => cursor_icon::CursorIcon::NeswResize,
        egui::CursorIcon::ResizeNwSe => cursor_icon::CursorIcon::NwseResize,
        egui::CursorIcon::ResizeVertical => cursor_icon::CursorIcon::RowResize,
        egui::CursorIcon::ResizeEast => cursor_icon::CursorIcon::EResize,
        egui::CursorIcon::ResizeSouthEast => cursor_icon::CursorIcon::SeResize,
        egui::CursorIcon::ResizeSouth => cursor_icon::CursorIcon::SResize,
        egui::CursorIcon::ResizeSouthWest => cursor_icon::CursorIcon::SwResize,
        egui::CursorIcon::ResizeWest => cursor_icon::CursorIcon::WResize,
        egui::CursorIcon::ResizeNorthWest => cursor_icon::CursorIcon::NwResize,
        egui::CursorIcon::ResizeNorth => cursor_icon::CursorIcon::NResize,
        egui::CursorIcon::ResizeNorthEast => cursor_icon::CursorIcon::NeResize,
        egui::CursorIcon::ResizeColumn => cursor_icon::CursorIcon::ColResize,
        egui::CursorIcon::ResizeRow => cursor_icon::CursorIcon::RowResize,
        egui::CursorIcon::ZoomIn => cursor_icon::CursorIcon::ZoomIn,
        egui::CursorIcon::ZoomOut => cursor_icon::CursorIcon::ZoomOut,
    })
}
