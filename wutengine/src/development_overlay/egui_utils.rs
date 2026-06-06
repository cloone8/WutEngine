use glam::Vec2;
use wutengine_asset::assets::sampler::FilterMode;
use wutengine_asset::assets::sampler::WrapMode;
use wutengine_asset::assets::sampler::WrapModeType;
use wutengine_asset::assets::texture::TextureConfig;
use wutengine_asset::assets::texture::TextureFormat;

use crate::graphics::sampler::Sampler;
use crate::input;
use crate::util::map;
use crate::window::Window;

#[inline]
pub const fn to_egui_vec2(v: crate::math::Vec2) -> egui::Vec2 {
    egui::Vec2 { x: v.x, y: v.y }
}

#[inline]
pub const fn to_egui_pos2(v: crate::math::Vec2) -> egui::Pos2 {
    egui::Pos2 { x: v.x, y: v.y }
}

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

#[inline]
pub fn egui_image_bytes(image: &egui::epaint::ImageData) -> &[u8] {
    match image {
        egui::ImageData::Color(color_image) => bytemuck::cast_slice(&color_image.pixels),
    }
}

#[inline]
pub fn sampler_from_egui(options: &egui::TextureOptions) -> Sampler {
    let filtering = filter_mode_from_egui(options.magnification);
    let wrapping = wrap_mode_from_egui(options.wrap_mode);

    Sampler::new(filtering, wrapping)
}

#[inline]
pub fn filter_mode_from_egui(egui_mode: egui::TextureFilter) -> FilterMode {
    match egui_mode {
        egui::TextureFilter::Nearest => FilterMode::Nearest,
        egui::TextureFilter::Linear => FilterMode::Linear,
    }
}

#[inline]
pub fn wrap_mode_from_egui(egui_mode: egui::TextureWrapMode) -> WrapModeType {
    match egui_mode {
        egui::TextureWrapMode::ClampToEdge => WrapModeType::Single(WrapMode::Clamp),
        egui::TextureWrapMode::Repeat => WrapModeType::Single(WrapMode::Repeat),
        egui::TextureWrapMode::MirroredRepeat => WrapModeType::Single(WrapMode::MirrorRepeat),
    }
}

fn add_mouse_button(
    button: u32,
    egui_button: egui::PointerButton,
    pos: egui::Pos2,
    modifiers: egui::Modifiers,
    events: &mut Vec<egui::Event>,
) {
    if input::mouse::button_pressed(None, button) {
        events.push(egui::Event::PointerButton {
            pos,
            button: egui_button,
            pressed: true,
            modifiers,
        });
    } else if input::mouse::button_released(None, button) {
        events.push(egui::Event::PointerButton {
            pos,
            button: egui_button,
            pressed: false,
            modifiers,
        });
    }
}

fn add_mouse_events(window: Window, events: &mut Vec<egui::Event>) {
    let modifiers = egui::Modifiers::NONE;

    let mouse_raw = input::mouse::pos_delta(None);

    if mouse_raw != Vec2::ZERO {
        events.push(egui::Event::MouseMoved(to_egui_vec2(mouse_raw)));
    }

    if let Some((pointer_window, pointer_pos)) = input::mouse::screen_pos(None)
        && pointer_window == window
    {
        let pointer_pos = to_egui_pos2(pointer_pos);
        events.push(egui::Event::PointerMoved(pointer_pos));

        add_mouse_button(
            input::mouse::BUTTON_LEFT,
            egui::PointerButton::Primary,
            pointer_pos,
            modifiers,
            events,
        );
        add_mouse_button(
            input::mouse::BUTTON_RIGHT,
            egui::PointerButton::Secondary,
            pointer_pos,
            modifiers,
            events,
        );
        add_mouse_button(
            input::mouse::BUTTON_MIDDLE,
            egui::PointerButton::Middle,
            pointer_pos,
            modifiers,
            events,
        );
    }

    let scroll_raw = input::mouse::scroll_delta(None);

    if scroll_raw != Vec2::ZERO {
        events.push(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Line,
            delta: to_egui_vec2(scroll_raw),
            phase: egui::TouchPhase::Move,
            modifiers,
        });
    }
}

pub fn gather_input(window: Window, surface_size: (u32, u32)) -> egui::RawInput {
    let sfc_rect = egui::Rect {
        min: egui::Pos2::ZERO,
        max: egui::Pos2::new(surface_size.0 as f32, surface_size.1 as f32),
    };

    let mut egui_events = Vec::new();

    add_mouse_events(window, &mut egui_events);

    egui::RawInput {
        viewport_id: egui::ViewportId::ROOT,
        viewports: map![
            egui::ViewportId::ROOT => egui::ViewportInfo {
                parent: None,
                title: Some("Development Overlay".to_string()),
                events: vec![],
                native_pixels_per_point: None,
                monitor_size: None,
                inner_rect: Some(sfc_rect),
                outer_rect: None,
                minimized: None,
                maximized: None,
                fullscreen: None,
                focused: Some(true),
                occluded: None
            }
        ],
        safe_area_insets: None,
        screen_rect: Some(sfc_rect),
        max_texture_side: None,
        time: Some(crate::time::unscaled_time64()),
        predicted_dt: 1.0 / 60.0,
        modifiers: egui::Modifiers::NONE,
        events: egui_events,
        hovered_files: vec![],
        dropped_files: vec![],
        focused: true,
        system_theme: None,
    }
}

/// A Rect in physical pixel space, used for setting clipping rectangles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScissorRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl ScissorRect {
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
