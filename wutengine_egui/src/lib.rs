#![doc = include_str!("../README.md")]

use std::sync::LazyLock;

use wutengine_asset::assets::sampler::FilterMode;
use wutengine_asset::assets::sampler::SerializedSampler;
use wutengine_asset::assets::sampler::WrapMode;
use wutengine_asset::assets::sampler::WrapModeType;
use wutengine_asset::assets::shader::SerializedShader;
use wutengine_asset::assets::shader::ShaderSource;
use wutengine_asset::assets::texture::TextureConfig;
use wutengine_asset::assets::texture::TextureFormat;
use wutengine_input::WindowIdentifier;
use wutengine_input::keyboard;
use wutengine_math::Vec2;
use wutengine_util::map;

mod key_mapping;

pub use key_mapping::*;

pub use egui;

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
    let wrapping = wrap_mode_from_egui(options.wrap_mode);

    SerializedSampler {
        filtering,
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

/// Checks a given mouse button for pressed/released events, and adds
/// them to the events list if applicable
fn add_mouse_button(
    button: u32,
    egui_button: egui::PointerButton,
    pos: egui::Pos2,
    modifiers: egui::Modifiers,
    events: &mut Vec<egui::Event>,
) {
    if wutengine_input::mouse::button_pressed(None, button) {
        events.push(egui::Event::PointerButton {
            pos,
            button: egui_button,
            pressed: true,
            modifiers,
        });
    } else if wutengine_input::mouse::button_released(None, button) {
        events.push(egui::Event::PointerButton {
            pos,
            button: egui_button,
            pressed: false,
            modifiers,
        });
    }
}

/// Checks for mouse events for the given window
fn add_mouse_events(
    window: WindowIdentifier,
    modifiers: egui::Modifiers,
    scale_factor: f32,
    events: &mut Vec<egui::Event>,
) {
    let mouse_raw = wutengine_input::mouse::pos_delta(None);

    if mouse_raw != Vec2::ZERO {
        events.push(egui::Event::MouseMoved(to_egui_vec2(mouse_raw)));
    }

    if let Some((pointer_window, pointer_pos)) = wutengine_input::mouse::screen_pos(None)
        && pointer_window == window
    {
        let pointer_pos = to_egui_pos2(pointer_pos, scale_factor);

        if mouse_raw != Vec2::ZERO {
            events.push(egui::Event::PointerMoved(pointer_pos));
        }

        add_mouse_button(
            wutengine_input::mouse::BUTTON_LEFT,
            egui::PointerButton::Primary,
            pointer_pos,
            modifiers,
            events,
        );
        add_mouse_button(
            wutengine_input::mouse::BUTTON_RIGHT,
            egui::PointerButton::Secondary,
            pointer_pos,
            modifiers,
            events,
        );
        add_mouse_button(
            wutengine_input::mouse::BUTTON_MIDDLE,
            egui::PointerButton::Middle,
            pointer_pos,
            modifiers,
            events,
        );
        add_mouse_button(
            wutengine_input::mouse::BUTTON_BACK,
            egui::PointerButton::Extra1,
            pointer_pos,
            modifiers,
            events,
        );
        add_mouse_button(
            wutengine_input::mouse::BUTTON_FORWARD,
            egui::PointerButton::Extra2,
            pointer_pos,
            modifiers,
            events,
        );
    }

    let scroll_raw = wutengine_input::mouse::scroll_delta(None);

    if scroll_raw != Vec2::ZERO {
        events.push(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Line,
            delta: to_egui_vec2(scroll_raw),
            phase: egui::TouchPhase::Move,
            modifiers,
        });
    }
}

/// Gathers the currently held modifier keys
fn gather_modifiers() -> egui::Modifiers {
    use wutengine_input::keyboard::key_held;

    const IS_MACOS: bool = cfg!(any(target_os = "macos", target_os = "ios"));

    let alt = key_held(None, keyboard::Key::AltLeft) || key_held(None, keyboard::Key::AltRight);

    let ctrl =
        key_held(None, keyboard::Key::ControlLeft) || key_held(None, keyboard::Key::ControlRight);

    let shift =
        key_held(None, keyboard::Key::ShiftLeft) || key_held(None, keyboard::Key::ShiftRight);

    let mac_cmd = IS_MACOS
        && (key_held(None, keyboard::Key::SuperLeft) || key_held(None, keyboard::Key::SuperRight));

    let command = if IS_MACOS { mac_cmd } else { ctrl };

    egui::Modifiers {
        alt,
        ctrl,
        shift,
        mac_cmd,
        command,
    }
}

/// Winit also sends control characters as text, so we ignore those
/// Ignore those.
/// We also ignore '\r', '\n', '\t'.
/// Newlines are handled by the `Key::Enter` event.
fn is_printable_char(chr: char) -> bool {
    let is_in_private_use_area = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);

    !is_in_private_use_area && !chr.is_ascii_control()
}

/// Adds keyboard events to the given events list. Returns the currently held modifier keys
fn add_keyboard_events(events: &mut Vec<egui::Event>) -> egui::Modifiers {
    let modifiers = gather_modifiers();

    let logical_inputs = wutengine_input::keyboard::logical_inputs(None);

    for logical_input in logical_inputs {
        let event = match logical_input {
            keyboard::LogicalInput::Pressed(logical_key) => {
                let Some(key) = wutengine_to_egui_key(logical_key) else {
                    continue;
                };

                egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers,
                }
            }
            keyboard::LogicalInput::Text(txt) => {
                let is_cmd = modifiers.ctrl || modifiers.command || modifiers.mac_cmd;

                if is_cmd || txt.chars().any(|c| !is_printable_char(c)) {
                    continue;
                }

                egui::Event::Text(txt)
            }
            keyboard::LogicalInput::Released(logical_key) => {
                let Some(key) = wutengine_to_egui_key(logical_key) else {
                    continue;
                };

                egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers,
                }
            }
        };

        events.push(event);
    }

    modifiers
}

/// Returns the input required to run [egui] for a frame
pub fn gather_input(
    window: WindowIdentifier,
    real_time_secs: f64,
    scale_factor: f32,
    surface_points: (f32, f32),
) -> egui::RawInput {
    let sfc_rect = egui::Rect {
        min: egui::Pos2::ZERO,
        max: egui::Pos2::new(surface_points.0, surface_points.1),
    };

    let mut egui_events = Vec::new();

    let modifiers = add_keyboard_events(&mut egui_events);

    add_mouse_events(window, modifiers, scale_factor, &mut egui_events);

    if !egui_events.is_empty() {
        log::trace!("Sending events: {:#?}", egui_events);
    }

    egui::RawInput {
        viewport_id: egui::ViewportId::ROOT,
        viewports: map![
            egui::ViewportId::ROOT => egui::ViewportInfo {
                parent: None,
                title: Some("Development Overlay".to_string()),
                events: vec![],
                native_pixels_per_point: Some(scale_factor),
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
        time: Some(real_time_secs),
        predicted_dt: 1.0 / 60.0,
        modifiers,
        events: egui_events,
        hovered_files: vec![],
        dropped_files: vec![],
        focused: true,
        system_theme: None,
    }
}

/// A shader that can be used for rendering the egui UI
pub static EGUI_SHADER: LazyLock<SerializedShader> = LazyLock::new(|| {
    let descriptor = include_str!("egui.json");
    let source = include_str!("egui.wgsl");

    let mut shader =
        serde_json::from_str::<SerializedShader>(descriptor).expect("Could not get egui shader");
    shader.source = ShaderSource::Inline {
        content: source.to_owned(),
    };

    shader
});
