use crate::winit::keyboard::KeyCode;

/// Converts a Winit keycode to a usize, for indexing into an array
pub(crate) const fn winit_keycode_to_usize(key: KeyCode) -> usize {
    match key {
        KeyCode::Backquote => 0,
        KeyCode::Backslash => 1,
        KeyCode::BracketLeft => 2,
        KeyCode::BracketRight => 3,
        KeyCode::Comma => 4,
        KeyCode::Digit0 => 5,
        KeyCode::Digit1 => 6,
        KeyCode::Digit2 => 7,
        KeyCode::Digit3 => 8,
        KeyCode::Digit4 => 9,
        KeyCode::Digit5 => 10,
        KeyCode::Digit6 => 11,
        KeyCode::Digit7 => 12,
        KeyCode::Digit8 => 13,
        KeyCode::Digit9 => 14,
        KeyCode::Equal => 15,
        KeyCode::IntlBackslash => 16,
        KeyCode::IntlRo => 17,
        KeyCode::IntlYen => 18,
        KeyCode::KeyA => 19,
        KeyCode::KeyB => 20,
        KeyCode::KeyC => 21,
        KeyCode::KeyD => 22,
        KeyCode::KeyE => 23,
        KeyCode::KeyF => 24,
        KeyCode::KeyG => 25,
        KeyCode::KeyH => 26,
        KeyCode::KeyI => 27,
        KeyCode::KeyJ => 28,
        KeyCode::KeyK => 29,
        KeyCode::KeyL => 30,
        KeyCode::KeyM => 31,
        KeyCode::KeyN => 32,
        KeyCode::KeyO => 33,
        KeyCode::KeyP => 34,
        KeyCode::KeyQ => 35,
        KeyCode::KeyR => 36,
        KeyCode::KeyS => 37,
        KeyCode::KeyT => 38,
        KeyCode::KeyU => 39,
        KeyCode::KeyV => 40,
        KeyCode::KeyW => 41,
        KeyCode::KeyX => 42,
        KeyCode::KeyY => 43,
        KeyCode::KeyZ => 44,
        KeyCode::Minus => 45,
        KeyCode::Period => 46,
        KeyCode::Quote => 47,
        KeyCode::Semicolon => 48,
        KeyCode::Slash => 49,
        KeyCode::AltLeft => 50,
        KeyCode::AltRight => 51,
        KeyCode::Backspace => 52,
        KeyCode::CapsLock => 53,
        KeyCode::ContextMenu => 54,
        KeyCode::ControlLeft => 55,
        KeyCode::ControlRight => 56,
        KeyCode::Enter => 57,
        KeyCode::SuperLeft => 58,
        KeyCode::SuperRight => 59,
        KeyCode::ShiftLeft => 60,
        KeyCode::ShiftRight => 61,
        KeyCode::Space => 62,
        KeyCode::Tab => 63,
        KeyCode::Convert => 64,
        KeyCode::KanaMode => 65,
        KeyCode::Lang1 => 66,
        KeyCode::Lang2 => 67,
        KeyCode::Lang3 => 68,
        KeyCode::Lang4 => 69,
        KeyCode::Lang5 => 70,
        KeyCode::NonConvert => 71,
        KeyCode::Delete => 72,
        KeyCode::End => 73,
        KeyCode::Help => 74,
        KeyCode::Home => 75,
        KeyCode::Insert => 76,
        KeyCode::PageDown => 77,
        KeyCode::PageUp => 78,
        KeyCode::ArrowDown => 79,
        KeyCode::ArrowLeft => 80,
        KeyCode::ArrowRight => 81,
        KeyCode::ArrowUp => 82,
        KeyCode::NumLock => 83,
        KeyCode::Numpad0 => 84,
        KeyCode::Numpad1 => 85,
        KeyCode::Numpad2 => 86,
        KeyCode::Numpad3 => 87,
        KeyCode::Numpad4 => 88,
        KeyCode::Numpad5 => 89,
        KeyCode::Numpad6 => 90,
        KeyCode::Numpad7 => 91,
        KeyCode::Numpad8 => 92,
        KeyCode::Numpad9 => 93,
        KeyCode::NumpadAdd => 94,
        KeyCode::NumpadBackspace => 95,
        KeyCode::NumpadClear => 96,
        KeyCode::NumpadClearEntry => 97,
        KeyCode::NumpadComma => 98,
        KeyCode::NumpadDecimal => 99,
        KeyCode::NumpadDivide => 100,
        KeyCode::NumpadEnter => 101,
        KeyCode::NumpadEqual => 102,
        KeyCode::NumpadHash => 103,
        KeyCode::NumpadMemoryAdd => 104,
        KeyCode::NumpadMemoryClear => 105,
        KeyCode::NumpadMemoryRecall => 106,
        KeyCode::NumpadMemoryStore => 107,
        KeyCode::NumpadMemorySubtract => 108,
        KeyCode::NumpadMultiply => 109,
        KeyCode::NumpadParenLeft => 110,
        KeyCode::NumpadParenRight => 111,
        KeyCode::NumpadStar => 112,
        KeyCode::NumpadSubtract => 113,
        KeyCode::Escape => 114,
        KeyCode::Fn => 115,
        KeyCode::FnLock => 116,
        KeyCode::PrintScreen => 117,
        KeyCode::ScrollLock => 118,
        KeyCode::Pause => 119,
        KeyCode::BrowserBack => 120,
        KeyCode::BrowserFavorites => 121,
        KeyCode::BrowserForward => 122,
        KeyCode::BrowserHome => 123,
        KeyCode::BrowserRefresh => 124,
        KeyCode::BrowserSearch => 125,
        KeyCode::BrowserStop => 126,
        KeyCode::Eject => 127,
        KeyCode::LaunchApp1 => 128,
        KeyCode::LaunchApp2 => 129,
        KeyCode::LaunchMail => 130,
        KeyCode::MediaPlayPause => 131,
        KeyCode::MediaSelect => 132,
        KeyCode::MediaStop => 133,
        KeyCode::MediaTrackNext => 134,
        KeyCode::MediaTrackPrevious => 135,
        KeyCode::Power => 136,
        KeyCode::Sleep => 137,
        KeyCode::AudioVolumeDown => 138,
        KeyCode::AudioVolumeMute => 139,
        KeyCode::AudioVolumeUp => 140,
        KeyCode::WakeUp => 141,
        KeyCode::Meta => 142,
        KeyCode::Hyper => 143,
        KeyCode::Turbo => 144,
        KeyCode::Abort => 145,
        KeyCode::Resume => 146,
        KeyCode::Suspend => 147,
        KeyCode::Again => 148,
        KeyCode::Copy => 149,
        KeyCode::Cut => 150,
        KeyCode::Find => 151,
        KeyCode::Open => 152,
        KeyCode::Paste => 153,
        KeyCode::Props => 154,
        KeyCode::Select => 155,
        KeyCode::Undo => 156,
        KeyCode::Hiragana => 157,
        KeyCode::Katakana => 158,
        KeyCode::F1 => 159,
        KeyCode::F2 => 160,
        KeyCode::F3 => 161,
        KeyCode::F4 => 162,
        KeyCode::F5 => 163,
        KeyCode::F6 => 164,
        KeyCode::F7 => 165,
        KeyCode::F8 => 166,
        KeyCode::F9 => 167,
        KeyCode::F10 => 168,
        KeyCode::F11 => 169,
        KeyCode::F12 => 170,
        KeyCode::F13 => 171,
        KeyCode::F14 => 172,
        KeyCode::F15 => 173,
        KeyCode::F16 => 174,
        KeyCode::F17 => 175,
        KeyCode::F18 => 176,
        KeyCode::F19 => 177,
        KeyCode::F20 => 178,
        KeyCode::F21 => 179,
        KeyCode::F22 => 180,
        KeyCode::F23 => 181,
        KeyCode::F24 => 182,
        KeyCode::F25 => 183,
        KeyCode::F26 => 184,
        KeyCode::F27 => 185,
        KeyCode::F28 => 186,
        KeyCode::F29 => 187,
        KeyCode::F30 => 188,
        KeyCode::F31 => 189,
        KeyCode::F32 => 190,
        KeyCode::F33 => 191,
        KeyCode::F34 => 192,
        KeyCode::F35 => 193,
        _ => panic!("Unknown KeyCode"),
    }
}
