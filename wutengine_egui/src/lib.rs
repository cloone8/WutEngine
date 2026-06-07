#![doc = include_str!("../README.md")]

pub use egui;
use wutengine_asset::assets::sampler::FilterMode;
use wutengine_asset::assets::sampler::SerializedSampler;
use wutengine_asset::assets::sampler::WrapMode;
use wutengine_asset::assets::sampler::WrapModeType;
use wutengine_asset::assets::texture::TextureConfig;
use wutengine_asset::assets::texture::TextureFormat;
use wutengine_input::WindowIdentifier;
use wutengine_input::keyboard;
use wutengine_math::Vec2;
use wutengine_util::map;

use wutengine_input as input;

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

fn add_mouse_events(
    window: WindowIdentifier,
    modifiers: egui::Modifiers,
    scale_factor: f32,
    events: &mut Vec<egui::Event>,
) {
    let mouse_raw = input::mouse::pos_delta(None);

    if mouse_raw != Vec2::ZERO {
        events.push(egui::Event::MouseMoved(to_egui_vec2(mouse_raw)));
    }

    if let Some((pointer_window, pointer_pos)) = input::mouse::screen_pos(None)
        && pointer_window == window
    {
        let pointer_pos = to_egui_pos2(pointer_pos, scale_factor);

        if mouse_raw != Vec2::ZERO {
            events.push(egui::Event::PointerMoved(pointer_pos));
        }

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

fn gather_modifiers() -> egui::Modifiers {
    use input::keyboard::key_held;

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

fn add_keyboard_events(events: &mut Vec<egui::Event>) -> egui::Modifiers {
    let modifiers = gather_modifiers();

    let logical_inputs = input::keyboard::logical_inputs(None);

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

/// Attempts to map a WutEngine [LogicalKey](keyboard::LogicalKey) to an [egui::Key]
#[inline]
#[expect(clippy::too_many_lines, reason = "Big match")]
pub fn wutengine_to_egui_key(key: keyboard::LogicalKey) -> Option<egui::Key> {
    match key {
        keyboard::LogicalKey::Character(c) => match c {
            '⏷' => Some(egui::Key::ArrowDown),
            '⏴' => Some(egui::Key::ArrowLeft),
            '⏵' => Some(egui::Key::ArrowRight),
            '⏶' => Some(egui::Key::ArrowUp),
            ' ' => Some(egui::Key::Space),
            ':' => Some(egui::Key::Colon),
            ',' => Some(egui::Key::Comma),
            '-' => Some(egui::Key::Minus),
            '.' => Some(egui::Key::Period),
            '+' => Some(egui::Key::Plus),
            '=' => Some(egui::Key::Equals),
            ';' => Some(egui::Key::Semicolon),
            '\\' => Some(egui::Key::Backslash),
            '/' => Some(egui::Key::Slash),
            '|' => Some(egui::Key::Pipe),
            '?' => Some(egui::Key::Questionmark),
            '!' => Some(egui::Key::Exclamationmark),
            '[' => Some(egui::Key::OpenBracket),
            ']' => Some(egui::Key::CloseBracket),
            '{' => Some(egui::Key::OpenCurlyBracket),
            '}' => Some(egui::Key::CloseCurlyBracket),
            '`' => Some(egui::Key::Backtick),
            '\'' => Some(egui::Key::Quote),

            '0' => Some(egui::Key::Num0),
            '1' => Some(egui::Key::Num1),
            '2' => Some(egui::Key::Num2),
            '3' => Some(egui::Key::Num3),
            '4' => Some(egui::Key::Num4),
            '5' => Some(egui::Key::Num5),
            '6' => Some(egui::Key::Num6),
            '7' => Some(egui::Key::Num7),
            '8' => Some(egui::Key::Num8),
            '9' => Some(egui::Key::Num9),

            'a' | 'A' => Some(egui::Key::A),
            'b' | 'B' => Some(egui::Key::B),
            'c' | 'C' => Some(egui::Key::C),
            'd' | 'D' => Some(egui::Key::D),
            'e' | 'E' => Some(egui::Key::E),
            'f' | 'F' => Some(egui::Key::F),
            'g' | 'G' => Some(egui::Key::G),
            'h' | 'H' => Some(egui::Key::H),
            'i' | 'I' => Some(egui::Key::I),
            'j' | 'J' => Some(egui::Key::J),
            'k' | 'K' => Some(egui::Key::K),
            'l' | 'L' => Some(egui::Key::L),
            'm' | 'M' => Some(egui::Key::M),
            'n' | 'N' => Some(egui::Key::N),
            'o' | 'O' => Some(egui::Key::O),
            'p' | 'P' => Some(egui::Key::P),
            'q' | 'Q' => Some(egui::Key::Q),
            'r' | 'R' => Some(egui::Key::R),
            's' | 'S' => Some(egui::Key::S),
            't' | 'T' => Some(egui::Key::T),
            'u' | 'U' => Some(egui::Key::U),
            'v' | 'V' => Some(egui::Key::V),
            'w' | 'W' => Some(egui::Key::W),
            'x' | 'X' => Some(egui::Key::X),
            'y' | 'Y' => Some(egui::Key::Y),
            'z' | 'Z' => Some(egui::Key::Z),
            _ => None,
        },
        keyboard::LogicalKey::String(s) => egui::Key::from_name(s.as_str()),
        keyboard::LogicalKey::Named(logical_named) => match logical_named {
            keyboard::LogicalNamed::Alt => None,
            keyboard::LogicalNamed::AltGraph => None,
            keyboard::LogicalNamed::CapsLock => None,
            keyboard::LogicalNamed::Control => None,
            keyboard::LogicalNamed::Fn => None,
            keyboard::LogicalNamed::FnLock => None,
            keyboard::LogicalNamed::NumLock => None,
            keyboard::LogicalNamed::ScrollLock => None,
            keyboard::LogicalNamed::Shift => None,
            keyboard::LogicalNamed::Symbol => None,
            keyboard::LogicalNamed::SymbolLock => None,
            keyboard::LogicalNamed::Meta => None,
            keyboard::LogicalNamed::Hyper => None,
            keyboard::LogicalNamed::Super => None,
            keyboard::LogicalNamed::Enter => Some(egui::Key::Enter),
            keyboard::LogicalNamed::Tab => Some(egui::Key::Tab),
            keyboard::LogicalNamed::Space => Some(egui::Key::Space),
            keyboard::LogicalNamed::ArrowDown => Some(egui::Key::ArrowDown),
            keyboard::LogicalNamed::ArrowLeft => Some(egui::Key::ArrowLeft),
            keyboard::LogicalNamed::ArrowRight => Some(egui::Key::ArrowRight),
            keyboard::LogicalNamed::ArrowUp => Some(egui::Key::ArrowUp),
            keyboard::LogicalNamed::End => Some(egui::Key::End),
            keyboard::LogicalNamed::Home => Some(egui::Key::Home),
            keyboard::LogicalNamed::PageDown => Some(egui::Key::PageDown),
            keyboard::LogicalNamed::PageUp => Some(egui::Key::PageUp),
            keyboard::LogicalNamed::Backspace => Some(egui::Key::Backspace),
            keyboard::LogicalNamed::Clear => None,
            keyboard::LogicalNamed::Copy => Some(egui::Key::Copy),
            keyboard::LogicalNamed::CrSel => None,
            keyboard::LogicalNamed::Cut => Some(egui::Key::Cut),
            keyboard::LogicalNamed::Delete => Some(egui::Key::Delete),
            keyboard::LogicalNamed::EraseEof => None,
            keyboard::LogicalNamed::ExSel => None,
            keyboard::LogicalNamed::Insert => Some(egui::Key::Insert),
            keyboard::LogicalNamed::Paste => Some(egui::Key::Paste),
            keyboard::LogicalNamed::Redo => None,
            keyboard::LogicalNamed::Undo => None,
            keyboard::LogicalNamed::Accept => None,
            keyboard::LogicalNamed::Again => None,
            keyboard::LogicalNamed::Attn => None,
            keyboard::LogicalNamed::Cancel => None,
            keyboard::LogicalNamed::ContextMenu => None,
            keyboard::LogicalNamed::Escape => Some(egui::Key::Escape),
            keyboard::LogicalNamed::Execute => None,
            keyboard::LogicalNamed::Find => None,
            keyboard::LogicalNamed::Help => None,
            keyboard::LogicalNamed::Pause => None,
            keyboard::LogicalNamed::Play => None,
            keyboard::LogicalNamed::Props => None,
            keyboard::LogicalNamed::Select => None,
            keyboard::LogicalNamed::ZoomIn => None,
            keyboard::LogicalNamed::ZoomOut => None,
            keyboard::LogicalNamed::BrightnessDown => None,
            keyboard::LogicalNamed::BrightnessUp => None,
            keyboard::LogicalNamed::Eject => None,
            keyboard::LogicalNamed::LogOff => None,
            keyboard::LogicalNamed::Power => None,
            keyboard::LogicalNamed::PowerOff => None,
            keyboard::LogicalNamed::PrintScreen => None,
            keyboard::LogicalNamed::Hibernate => None,
            keyboard::LogicalNamed::Standby => None,
            keyboard::LogicalNamed::WakeUp => None,
            keyboard::LogicalNamed::AllCandidates => None,
            keyboard::LogicalNamed::Alphanumeric => None,
            keyboard::LogicalNamed::CodeInput => None,
            keyboard::LogicalNamed::Compose => None,
            keyboard::LogicalNamed::Convert => None,
            keyboard::LogicalNamed::FinalMode => None,
            keyboard::LogicalNamed::GroupFirst => None,
            keyboard::LogicalNamed::GroupLast => None,
            keyboard::LogicalNamed::GroupNext => None,
            keyboard::LogicalNamed::GroupPrevious => None,
            keyboard::LogicalNamed::ModeChange => None,
            keyboard::LogicalNamed::NextCandidate => None,
            keyboard::LogicalNamed::NonConvert => None,
            keyboard::LogicalNamed::PreviousCandidate => None,
            keyboard::LogicalNamed::Process => None,
            keyboard::LogicalNamed::SingleCandidate => None,
            keyboard::LogicalNamed::HangulMode => None,
            keyboard::LogicalNamed::HanjaMode => None,
            keyboard::LogicalNamed::JunjaMode => None,
            keyboard::LogicalNamed::Eisu => None,
            keyboard::LogicalNamed::Hankaku => None,
            keyboard::LogicalNamed::Hiragana => None,
            keyboard::LogicalNamed::HiraganaKatakana => None,
            keyboard::LogicalNamed::KanaMode => None,
            keyboard::LogicalNamed::KanjiMode => None,
            keyboard::LogicalNamed::Katakana => None,
            keyboard::LogicalNamed::Romaji => None,
            keyboard::LogicalNamed::Zenkaku => None,
            keyboard::LogicalNamed::ZenkakuHankaku => None,
            keyboard::LogicalNamed::Soft1 => None,
            keyboard::LogicalNamed::Soft2 => None,
            keyboard::LogicalNamed::Soft3 => None,
            keyboard::LogicalNamed::Soft4 => None,
            keyboard::LogicalNamed::ChannelDown => None,
            keyboard::LogicalNamed::ChannelUp => None,
            keyboard::LogicalNamed::Close => None,
            keyboard::LogicalNamed::MailForward => None,
            keyboard::LogicalNamed::MailReply => None,
            keyboard::LogicalNamed::MailSend => None,
            keyboard::LogicalNamed::MediaClose => None,
            keyboard::LogicalNamed::MediaFastForward => None,
            keyboard::LogicalNamed::MediaPause => None,
            keyboard::LogicalNamed::MediaPlay => None,
            keyboard::LogicalNamed::MediaPlayPause => None,
            keyboard::LogicalNamed::MediaRecord => None,
            keyboard::LogicalNamed::MediaRewind => None,
            keyboard::LogicalNamed::MediaStop => None,
            keyboard::LogicalNamed::MediaTrackNext => None,
            keyboard::LogicalNamed::MediaTrackPrevious => None,
            keyboard::LogicalNamed::New => None,
            keyboard::LogicalNamed::Open => None,
            keyboard::LogicalNamed::Print => None,
            keyboard::LogicalNamed::Save => None,
            keyboard::LogicalNamed::SpellCheck => None,
            keyboard::LogicalNamed::Key11 => None,
            keyboard::LogicalNamed::Key12 => None,
            keyboard::LogicalNamed::AudioBalanceLeft => None,
            keyboard::LogicalNamed::AudioBalanceRight => None,
            keyboard::LogicalNamed::AudioBassBoostDown => None,
            keyboard::LogicalNamed::AudioBassBoostToggle => None,
            keyboard::LogicalNamed::AudioBassBoostUp => None,
            keyboard::LogicalNamed::AudioFaderFront => None,
            keyboard::LogicalNamed::AudioFaderRear => None,
            keyboard::LogicalNamed::AudioSurroundModeNext => None,
            keyboard::LogicalNamed::AudioTrebleDown => None,
            keyboard::LogicalNamed::AudioTrebleUp => None,
            keyboard::LogicalNamed::AudioVolumeDown => None,
            keyboard::LogicalNamed::AudioVolumeUp => None,
            keyboard::LogicalNamed::AudioVolumeMute => None,
            keyboard::LogicalNamed::MicrophoneToggle => None,
            keyboard::LogicalNamed::MicrophoneVolumeDown => None,
            keyboard::LogicalNamed::MicrophoneVolumeUp => None,
            keyboard::LogicalNamed::MicrophoneVolumeMute => None,
            keyboard::LogicalNamed::SpeechCorrectionList => None,
            keyboard::LogicalNamed::SpeechInputToggle => None,
            keyboard::LogicalNamed::LaunchApplication1 => None,
            keyboard::LogicalNamed::LaunchApplication2 => None,
            keyboard::LogicalNamed::LaunchCalendar => None,
            keyboard::LogicalNamed::LaunchContacts => None,
            keyboard::LogicalNamed::LaunchMail => None,
            keyboard::LogicalNamed::LaunchMediaPlayer => None,
            keyboard::LogicalNamed::LaunchMusicPlayer => None,
            keyboard::LogicalNamed::LaunchPhone => None,
            keyboard::LogicalNamed::LaunchScreenSaver => None,
            keyboard::LogicalNamed::LaunchSpreadsheet => None,
            keyboard::LogicalNamed::LaunchWebBrowser => None,
            keyboard::LogicalNamed::LaunchWebCam => None,
            keyboard::LogicalNamed::LaunchWordProcessor => None,
            keyboard::LogicalNamed::BrowserBack => Some(egui::Key::BrowserBack),
            keyboard::LogicalNamed::BrowserFavorites => None,
            keyboard::LogicalNamed::BrowserForward => None,
            keyboard::LogicalNamed::BrowserHome => None,
            keyboard::LogicalNamed::BrowserRefresh => None,
            keyboard::LogicalNamed::BrowserSearch => None,
            keyboard::LogicalNamed::BrowserStop => None,
            keyboard::LogicalNamed::AppSwitch => None,
            keyboard::LogicalNamed::Call => None,
            keyboard::LogicalNamed::Camera => None,
            keyboard::LogicalNamed::CameraFocus => None,
            keyboard::LogicalNamed::EndCall => None,
            keyboard::LogicalNamed::GoBack => None,
            keyboard::LogicalNamed::GoHome => None,
            keyboard::LogicalNamed::HeadsetHook => None,
            keyboard::LogicalNamed::LastNumberRedial => None,
            keyboard::LogicalNamed::Notification => None,
            keyboard::LogicalNamed::MannerMode => None,
            keyboard::LogicalNamed::VoiceDial => None,
            keyboard::LogicalNamed::TV => None,
            keyboard::LogicalNamed::TV3DMode => None,
            keyboard::LogicalNamed::TVAntennaCable => None,
            keyboard::LogicalNamed::TVAudioDescription => None,
            keyboard::LogicalNamed::TVAudioDescriptionMixDown => None,
            keyboard::LogicalNamed::TVAudioDescriptionMixUp => None,
            keyboard::LogicalNamed::TVContentsMenu => None,
            keyboard::LogicalNamed::TVDataService => None,
            keyboard::LogicalNamed::TVInput => None,
            keyboard::LogicalNamed::TVInputComponent1 => None,
            keyboard::LogicalNamed::TVInputComponent2 => None,
            keyboard::LogicalNamed::TVInputComposite1 => None,
            keyboard::LogicalNamed::TVInputComposite2 => None,
            keyboard::LogicalNamed::TVInputHDMI1 => None,
            keyboard::LogicalNamed::TVInputHDMI2 => None,
            keyboard::LogicalNamed::TVInputHDMI3 => None,
            keyboard::LogicalNamed::TVInputHDMI4 => None,
            keyboard::LogicalNamed::TVInputVGA1 => None,
            keyboard::LogicalNamed::TVMediaContext => None,
            keyboard::LogicalNamed::TVNetwork => None,
            keyboard::LogicalNamed::TVNumberEntry => None,
            keyboard::LogicalNamed::TVPower => None,
            keyboard::LogicalNamed::TVRadioService => None,
            keyboard::LogicalNamed::TVSatellite => None,
            keyboard::LogicalNamed::TVSatelliteBS => None,
            keyboard::LogicalNamed::TVSatelliteCS => None,
            keyboard::LogicalNamed::TVSatelliteToggle => None,
            keyboard::LogicalNamed::TVTerrestrialAnalog => None,
            keyboard::LogicalNamed::TVTerrestrialDigital => None,
            keyboard::LogicalNamed::TVTimer => None,
            keyboard::LogicalNamed::AVRInput => None,
            keyboard::LogicalNamed::AVRPower => None,
            keyboard::LogicalNamed::ColorF0Red => None,
            keyboard::LogicalNamed::ColorF1Green => None,
            keyboard::LogicalNamed::ColorF2Yellow => None,
            keyboard::LogicalNamed::ColorF3Blue => None,
            keyboard::LogicalNamed::ColorF4Grey => None,
            keyboard::LogicalNamed::ColorF5Brown => None,
            keyboard::LogicalNamed::ClosedCaptionToggle => None,
            keyboard::LogicalNamed::Dimmer => None,
            keyboard::LogicalNamed::DisplaySwap => None,
            keyboard::LogicalNamed::DVR => None,
            keyboard::LogicalNamed::Exit => None,
            keyboard::LogicalNamed::FavoriteClear0 => None,
            keyboard::LogicalNamed::FavoriteClear1 => None,
            keyboard::LogicalNamed::FavoriteClear2 => None,
            keyboard::LogicalNamed::FavoriteClear3 => None,
            keyboard::LogicalNamed::FavoriteRecall0 => None,
            keyboard::LogicalNamed::FavoriteRecall1 => None,
            keyboard::LogicalNamed::FavoriteRecall2 => None,
            keyboard::LogicalNamed::FavoriteRecall3 => None,
            keyboard::LogicalNamed::FavoriteStore0 => None,
            keyboard::LogicalNamed::FavoriteStore1 => None,
            keyboard::LogicalNamed::FavoriteStore2 => None,
            keyboard::LogicalNamed::FavoriteStore3 => None,
            keyboard::LogicalNamed::Guide => None,
            keyboard::LogicalNamed::GuideNextDay => None,
            keyboard::LogicalNamed::GuidePreviousDay => None,
            keyboard::LogicalNamed::Info => None,
            keyboard::LogicalNamed::InstantReplay => None,
            keyboard::LogicalNamed::Link => None,
            keyboard::LogicalNamed::ListProgram => None,
            keyboard::LogicalNamed::LiveContent => None,
            keyboard::LogicalNamed::Lock => None,
            keyboard::LogicalNamed::MediaApps => None,
            keyboard::LogicalNamed::MediaAudioTrack => None,
            keyboard::LogicalNamed::MediaLast => None,
            keyboard::LogicalNamed::MediaSkipBackward => None,
            keyboard::LogicalNamed::MediaSkipForward => None,
            keyboard::LogicalNamed::MediaStepBackward => None,
            keyboard::LogicalNamed::MediaStepForward => None,
            keyboard::LogicalNamed::MediaTopMenu => None,
            keyboard::LogicalNamed::NavigateIn => None,
            keyboard::LogicalNamed::NavigateNext => None,
            keyboard::LogicalNamed::NavigateOut => None,
            keyboard::LogicalNamed::NavigatePrevious => None,
            keyboard::LogicalNamed::NextFavoriteChannel => None,
            keyboard::LogicalNamed::NextUserProfile => None,
            keyboard::LogicalNamed::OnDemand => None,
            keyboard::LogicalNamed::Pairing => None,
            keyboard::LogicalNamed::PinPDown => None,
            keyboard::LogicalNamed::PinPMove => None,
            keyboard::LogicalNamed::PinPToggle => None,
            keyboard::LogicalNamed::PinPUp => None,
            keyboard::LogicalNamed::PlaySpeedDown => None,
            keyboard::LogicalNamed::PlaySpeedReset => None,
            keyboard::LogicalNamed::PlaySpeedUp => None,
            keyboard::LogicalNamed::RandomToggle => None,
            keyboard::LogicalNamed::RcLowBattery => None,
            keyboard::LogicalNamed::RecordSpeedNext => None,
            keyboard::LogicalNamed::RfBypass => None,
            keyboard::LogicalNamed::ScanChannelsToggle => None,
            keyboard::LogicalNamed::ScreenModeNext => None,
            keyboard::LogicalNamed::Settings => None,
            keyboard::LogicalNamed::SplitScreenToggle => None,
            keyboard::LogicalNamed::STBInput => None,
            keyboard::LogicalNamed::STBPower => None,
            keyboard::LogicalNamed::Subtitle => None,
            keyboard::LogicalNamed::Teletext => None,
            keyboard::LogicalNamed::VideoModeNext => None,
            keyboard::LogicalNamed::Wink => None,
            keyboard::LogicalNamed::ZoomToggle => None,
            keyboard::LogicalNamed::F1 => Some(egui::Key::F1),
            keyboard::LogicalNamed::F2 => Some(egui::Key::F2),
            keyboard::LogicalNamed::F3 => Some(egui::Key::F3),
            keyboard::LogicalNamed::F4 => Some(egui::Key::F4),
            keyboard::LogicalNamed::F5 => Some(egui::Key::F5),
            keyboard::LogicalNamed::F6 => Some(egui::Key::F6),
            keyboard::LogicalNamed::F7 => Some(egui::Key::F7),
            keyboard::LogicalNamed::F8 => Some(egui::Key::F8),
            keyboard::LogicalNamed::F9 => Some(egui::Key::F9),
            keyboard::LogicalNamed::F10 => Some(egui::Key::F10),
            keyboard::LogicalNamed::F11 => Some(egui::Key::F11),
            keyboard::LogicalNamed::F12 => Some(egui::Key::F12),
            keyboard::LogicalNamed::F13 => Some(egui::Key::F13),
            keyboard::LogicalNamed::F14 => Some(egui::Key::F14),
            keyboard::LogicalNamed::F15 => Some(egui::Key::F15),
            keyboard::LogicalNamed::F16 => Some(egui::Key::F16),
            keyboard::LogicalNamed::F17 => Some(egui::Key::F17),
            keyboard::LogicalNamed::F18 => Some(egui::Key::F18),
            keyboard::LogicalNamed::F19 => Some(egui::Key::F19),
            keyboard::LogicalNamed::F20 => Some(egui::Key::F20),
            keyboard::LogicalNamed::F21 => Some(egui::Key::F21),
            keyboard::LogicalNamed::F22 => Some(egui::Key::F22),
            keyboard::LogicalNamed::F23 => Some(egui::Key::F23),
            keyboard::LogicalNamed::F24 => Some(egui::Key::F24),
            keyboard::LogicalNamed::F25 => Some(egui::Key::F25),
            keyboard::LogicalNamed::F26 => Some(egui::Key::F26),
            keyboard::LogicalNamed::F27 => Some(egui::Key::F27),
            keyboard::LogicalNamed::F28 => Some(egui::Key::F28),
            keyboard::LogicalNamed::F29 => Some(egui::Key::F29),
            keyboard::LogicalNamed::F30 => Some(egui::Key::F30),
            keyboard::LogicalNamed::F31 => Some(egui::Key::F31),
            keyboard::LogicalNamed::F32 => Some(egui::Key::F32),
            keyboard::LogicalNamed::F33 => Some(egui::Key::F33),
            keyboard::LogicalNamed::F34 => Some(egui::Key::F34),
            keyboard::LogicalNamed::F35 => Some(egui::Key::F35),
        },
        keyboard::LogicalKey::Unknown(_) => None,
    }
}
