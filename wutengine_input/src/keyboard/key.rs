use winit::platform::scancode::PhysicalKeyExtScancode;
use wutengine_util_macro::VariantIndex;

use super::winit_native_keycode_to_u32;

/// A physical key. Note that these correspond to key _locations_, not actual logical inputs
///
/// Taken from [winit 0.30.13](https://github.com/rust-windowing/winit/tree/v0.30.13),
/// and modified to suit WutEngine APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, VariantIndex)]
#[index_repr(u32)]
pub enum Key {
    /// <kbd>`</kbd>
    Backquote,

    /// <kbd>\\</kbd>
    Backslash,

    /// <kbd>[</kbd> on a US keyboard.
    BracketLeft,

    /// <kbd>]</kbd> on a US keyboard.
    BracketRight,

    /// <kbd>,</kbd> on a US keyboard.
    Comma,
    /// <kbd>0</kbd> on a US keyboard.
    Digit0,
    /// <kbd>1</kbd> on a US keyboard.
    Digit1,
    /// <kbd>2</kbd> on a US keyboard.
    Digit2,
    /// <kbd>3</kbd> on a US keyboard.
    Digit3,
    /// <kbd>4</kbd> on a US keyboard.
    Digit4,
    /// <kbd>5</kbd> on a US keyboard.
    Digit5,
    /// <kbd>6</kbd> on a US keyboard.
    Digit6,
    /// <kbd>7</kbd> on a US keyboard.
    Digit7,
    /// <kbd>8</kbd> on a US keyboard.
    Digit8,
    /// <kbd>9</kbd> on a US keyboard.
    Digit9,
    /// <kbd>=</kbd> on a US keyboard.
    Equal,

    /// Located between the left <kbd>Shift</kbd> and <kbd>Z</kbd> keys.
    /// Labeled <kbd>\\</kbd> on a UK keyboard.
    IntlBackslash,

    /// Located between the <kbd>/</kbd> and right <kbd>Shift</kbd> keys.
    /// Labeled <kbd>\\</kbd> (ro) on a Japanese keyboard.
    IntlRo,

    /// Located between the <kbd>=</kbd> and <kbd>Backspace</kbd> keys.
    /// Labeled <kbd>¥</kbd> (yen) on a Japanese keyboard. <kbd>\\</kbd> on a
    /// Russian keyboard.
    IntlYen,

    /// <kbd>a</kbd> on a US keyboard.
    /// Labeled <kbd>q</kbd> on an AZERTY (e.g., French) keyboard.
    A,

    /// <kbd>b</kbd> on a US keyboard.
    B,

    /// <kbd>c</kbd> on a US keyboard.
    C,

    /// <kbd>d</kbd> on a US keyboard.
    D,

    /// <kbd>e</kbd> on a US keyboard.
    E,

    /// <kbd>f</kbd> on a US keyboard.
    F,

    /// <kbd>g</kbd> on a US keyboard.
    G,

    /// <kbd>h</kbd> on a US keyboard.
    H,

    /// <kbd>i</kbd> on a US keyboard.
    I,

    /// <kbd>j</kbd> on a US keyboard.
    J,

    /// <kbd>k</kbd> on a US keyboard.
    K,

    /// <kbd>l</kbd> on a US keyboard.
    L,

    /// <kbd>m</kbd> on a US keyboard.
    M,

    /// <kbd>n</kbd> on a US keyboard.
    N,

    /// <kbd>o</kbd> on a US keyboard.
    O,

    /// <kbd>p</kbd> on a US keyboard.
    P,

    /// <kbd>q</kbd> on a US keyboard.
    /// Labeled <kbd>a</kbd> on an AZERTY (e.g., French) keyboard.
    Q,

    /// <kbd>r</kbd> on a US keyboard.
    R,

    /// <kbd>s</kbd> on a US keyboard.
    S,

    /// <kbd>t</kbd> on a US keyboard.
    T,

    /// <kbd>u</kbd> on a US keyboard.
    U,

    /// <kbd>v</kbd> on a US keyboard.
    V,

    /// <kbd>w</kbd> on a US keyboard.
    /// Labeled <kbd>z</kbd> on an AZERTY (e.g., French) keyboard.
    W,

    /// <kbd>x</kbd> on a US keyboard.
    X,

    /// <kbd>y</kbd> on a US keyboard.
    /// Labeled <kbd>z</kbd> on a QWERTZ (e.g., German) keyboard.
    Y,

    /// <kbd>z</kbd> on a US keyboard.
    /// Labeled <kbd>w</kbd> on an AZERTY (e.g., French) keyboard, and <kbd>y</kbd> on a
    /// QWERTZ (e.g., German) keyboard.
    Z,

    /// <kbd>-</kbd> on a US keyboard.
    Minus,

    /// <kbd>.</kbd> on a US keyboard.
    Period,

    /// <kbd>'</kbd> on a US keyboard.
    Quote,

    /// <kbd>;</kbd> on a US keyboard.
    Semicolon,

    /// <kbd>/</kbd> on a US keyboard.
    Slash,

    /// <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
    AltLeft,

    /// <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
    /// This is labeled <kbd>AltGr</kbd> on many keyboard layouts.
    AltRight,

    /// <kbd>Backspace</kbd> or <kbd>⌫</kbd>.
    /// Labeled <kbd>Delete</kbd> on Apple keyboards.
    Backspace,

    /// <kbd>CapsLock</kbd> or <kbd>⇪</kbd>
    CapsLock,

    /// The application context menu key, which is typically found between the right
    /// <kbd>Super</kbd> key and the right <kbd>Control</kbd> key.
    ContextMenu,

    /// <kbd>Control</kbd> or <kbd>⌃</kbd>
    ControlLeft,

    /// <kbd>Control</kbd> or <kbd>⌃</kbd>
    ControlRight,

    /// <kbd>Enter</kbd> or <kbd>↵</kbd>. Labeled <kbd>Return</kbd> on Apple keyboards.
    Enter,

    /// The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
    SuperLeft,

    /// The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
    SuperRight,

    /// <kbd>Shift</kbd> or <kbd>⇧</kbd>
    ShiftLeft,

    /// <kbd>Shift</kbd> or <kbd>⇧</kbd>
    ShiftRight,

    /// <kbd> </kbd> (space)
    Space,

    /// <kbd>Tab</kbd> or <kbd>⇥</kbd>
    Tab,

    /// Japanese: <kbd>変</kbd> (henkan)
    Convert,

    /// Japanese: <kbd>カタカナ</kbd>/<kbd>ひらがな</kbd>/<kbd>ローマ字</kbd>
    /// (katakana/hiragana/romaji)
    KanaMode,

    /// Korean: HangulMode <kbd>한/영</kbd> (han/yeong)
    ///
    /// Japanese (Mac keyboard): <kbd>か</kbd> (kana)
    Lang1,

    /// Korean: Hanja <kbd>한</kbd> (hanja)
    ///
    /// Japanese (Mac keyboard): <kbd>英</kbd> (eisu)
    Lang2,

    /// Japanese (word-processing keyboard): Katakana
    Lang3,

    /// Japanese (word-processing keyboard): Hiragana
    Lang4,

    /// Japanese (word-processing keyboard): Zenkaku/Hankaku
    Lang5,

    /// Japanese: <kbd>無変換</kbd> (muhenkan)
    NonConvert,

    /// <kbd>⌦</kbd>. The forward delete key.
    /// Note that on Apple keyboards, the key labelled <kbd>Delete</kbd> on the main part of
    /// the keyboard is encoded as [`Backspace`].
    ///
    /// [`Backspace`]: Self::Backspace
    Delete,

    /// <kbd>Page Down</kbd>, <kbd>End</kbd>, or <kbd>↘</kbd>
    End,

    /// <kbd>Help</kbd>. Not present on standard PC keyboards.
    Help,

    /// <kbd>Home</kbd> or <kbd>↖</kbd>
    Home,

    /// <kbd>Insert</kbd> or <kbd>Ins</kbd>. Not present on Apple keyboards.
    Insert,

    /// <kbd>Page Down</kbd>, <kbd>PgDn</kbd>, or <kbd>⇟</kbd>
    PageDown,
    /// <kbd>Page Up</kbd>, <kbd>PgUp</kbd>, or <kbd>⇞</kbd>
    PageUp,

    /// <kbd>↓</kbd>
    ArrowDown,

    /// <kbd>←</kbd>
    ArrowLeft,

    /// <kbd>→</kbd>
    ArrowRight,

    /// <kbd>↑</kbd>
    ArrowUp,

    /// On the Mac, this is used for the numpad <kbd>Clear</kbd> key.
    NumLock,

    /// <kbd>0 Ins</kbd> on a keyboard. <kbd>0</kbd> on a phone or remote control
    Numpad0,

    /// <kbd>1 End</kbd> on a keyboard. <kbd>1</kbd> or <kbd>1 QZ</kbd> on a phone or remote
    /// control
    Numpad1,

    /// <kbd>2 ↓</kbd> on a keyboard. <kbd>2 ABC</kbd> on a phone or remote control
    Numpad2,

    /// <kbd>3 PgDn</kbd> on a keyboard. <kbd>3 DEF</kbd> on a phone or remote control
    Numpad3,

    /// <kbd>4 ←</kbd> on a keyboard. <kbd>4 GHI</kbd> on a phone or remote control
    Numpad4,

    /// <kbd>5</kbd> on a keyboard. <kbd>5 JKL</kbd> on a phone or remote control
    Numpad5,

    /// <kbd>6 →</kbd> on a keyboard. <kbd>6 MNO</kbd> on a phone or remote control
    Numpad6,

    /// <kbd>7 Home</kbd> on a keyboard. <kbd>7 PQRS</kbd> or <kbd>7 PRS</kbd> on a phone
    /// or remote control
    Numpad7,

    /// <kbd>8 ↑</kbd> on a keyboard. <kbd>8 TUV</kbd> on a phone or remote control
    Numpad8,

    /// <kbd>9 PgUp</kbd> on a keyboard. <kbd>9 WXYZ</kbd> or <kbd>9 WXY</kbd> on a phone
    /// or remote control
    Numpad9,

    /// <kbd>+</kbd>
    NumpadAdd,

    /// Found on the Microsoft Natural Keyboard.
    NumpadBackspace,

    /// <kbd>C</kbd> or <kbd>A</kbd> (All Clear). Also for use with numpads that have a
    /// <kbd>Clear</kbd> key that is separate from the <kbd>NumLock</kbd> key. On the Mac, the
    /// numpad <kbd>Clear</kbd> key is encoded as [`NumLock`].
    ///
    /// [`NumLock`]: Self::NumLock
    NumpadClear,

    /// <kbd>C</kbd> (Clear Entry)
    NumpadClearEntry,

    /// <kbd>,</kbd> (thousands separator). For locales where the thousands separator
    /// is a "." (e.g., Brazil), this key may generate a <kbd>.</kbd>.
    NumpadComma,

    /// <kbd>. Del</kbd>. For locales where the decimal separator is "," (e.g.,
    /// Brazil), this key may generate a <kbd>,</kbd>.
    NumpadDecimal,

    /// <kbd>/</kbd>
    NumpadDivide,

    /// <kbd>enter</kbd> on the numpad
    NumpadEnter,

    /// <kbd>=</kbd>
    NumpadEqual,

    /// <kbd>#</kbd> on a phone or remote control device. This key is typically found
    /// below the <kbd>9</kbd> key and to the right of the <kbd>0</kbd> key.
    NumpadHash,

    /// <kbd>M</kbd> Add current entry to the value stored in memory.
    NumpadMemoryAdd,

    /// <kbd>M</kbd> Clear the value stored in memory.
    NumpadMemoryClear,

    /// <kbd>M</kbd> Replace the current entry with the value stored in memory.
    NumpadMemoryRecall,

    /// <kbd>M</kbd> Replace the value stored in memory with the current entry.
    NumpadMemoryStore,

    /// <kbd>M</kbd> Subtract current entry from the value stored in memory.
    NumpadMemorySubtract,

    /// <kbd>*</kbd> on a keyboard. For use with numpads that provide mathematical
    /// operations (<kbd>+</kbd>, <kbd>-</kbd> <kbd>*</kbd> and <kbd>/</kbd>).
    ///
    /// Use `NumpadStar` for the <kbd>*</kbd> key on phones and remote controls.
    NumpadMultiply,

    /// <kbd>(</kbd> Found on the Microsoft Natural Keyboard.
    NumpadParenLeft,

    /// <kbd>)</kbd> Found on the Microsoft Natural Keyboard.
    NumpadParenRight,

    /// <kbd>*</kbd> on a phone or remote control device.
    ///
    /// This key is typically found below the <kbd>7</kbd> key and to the left of
    /// the <kbd>0</kbd> key.
    ///
    /// Use <kbd>"NumpadMultiply"</kbd> for the <kbd>*</kbd> key on
    /// numeric keypads.
    NumpadStar,

    /// <kbd>-</kbd>
    NumpadSubtract,

    /// <kbd>Esc</kbd> or <kbd>⎋</kbd>
    Escape,

    /// <kbd>Fn</kbd> This is typically a hardware key that does not generate a separate code.
    Fn,

    /// <kbd>FLock</kbd> or <kbd>FnLock</kbd>. Function Lock key. Found on the Microsoft
    /// Natural Keyboard.
    FnLock,

    /// <kbd>PrtScr SysRq</kbd> or <kbd>Print Screen</kbd>
    PrintScreen,

    /// <kbd>Scroll Lock</kbd>
    ScrollLock,

    /// <kbd>Pause Break</kbd>
    Pause,

    /// Some laptops place this key to the left of the <kbd>↑</kbd> key.
    ///
    /// This also the "back" button (triangle) on Android.
    BrowserBack,

    /// Browser favourite key. Only supported by some keyboards
    BrowserFavorites,

    /// Some laptops place this key to the right of the <kbd>↑</kbd> key.
    BrowserForward,

    /// The "home" button on Android.
    BrowserHome,

    /// Legacy browser key
    BrowserRefresh,

    /// Legacy browser key
    BrowserSearch,

    /// Legacy browser key
    BrowserStop,

    /// <kbd>Eject</kbd> or <kbd>⏏</kbd>. This key is placed in the function section on some Apple
    /// keyboards.
    Eject,

    /// Sometimes labelled <kbd>My Computer</kbd> on the keyboard
    LaunchApp1,

    /// Sometimes labelled <kbd>Calculator</kbd> on the keyboard
    LaunchApp2,

    /// Legacy app key
    LaunchMail,

    /// Play/pause button
    MediaPlayPause,

    /// Media select button
    MediaSelect,

    /// Media stop button
    MediaStop,

    /// Media next button
    MediaTrackNext,

    /// Media previous button
    MediaTrackPrevious,

    /// This key is placed in the function section on some Apple keyboards, replacing the
    /// <kbd>Eject</kbd> key.
    Power,

    /// Sleep button
    Sleep,

    /// Volume down button
    AudioVolumeDown,

    /// Mute button
    AudioVolumeMute,

    /// Volume up button
    AudioVolumeUp,

    /// Wake button
    WakeUp,

    /// Legacy modifier key. Also called "Super" in certain places.
    Meta,
    /// Legacy modifier key.
    Hyper,
    /// Legacy modifier key.
    Turbo,
    /// Legacy modifier key.
    Abort,
    /// Legacy modifier key.
    Resume,
    /// Legacy modifier key.
    Suspend,
    /// Found on Sun’s USB keyboard.
    Again,
    /// Found on Sun’s USB keyboard.
    Copy,
    /// Found on Sun’s USB keyboard.
    Cut,
    /// Found on Sun’s USB keyboard.
    Find,
    /// Found on Sun’s USB keyboard.
    Open,
    /// Found on Sun’s USB keyboard.
    Paste,
    /// Found on Sun’s USB keyboard.
    Props,
    /// Found on Sun’s USB keyboard.
    Select,
    /// Found on Sun’s USB keyboard.
    Undo,
    /// Use for dedicated <kbd>ひらがな</kbd> key found on some Japanese word processing keyboards.
    Hiragana,
    /// Use for dedicated <kbd>カタカナ</kbd> key found on some Japanese word processing keyboards.
    Katakana,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F1,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F2,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F3,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F4,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F5,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F6,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F7,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F8,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F9,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F10,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F11,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F12,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F13,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F14,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F15,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F16,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F17,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F18,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F19,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F20,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F21,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F22,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F23,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F24,
    /// General-purpose function key.
    F25,
    /// General-purpose function key.
    F26,
    /// General-purpose function key.
    F27,
    /// General-purpose function key.
    F28,
    /// General-purpose function key.
    F29,
    /// General-purpose function key.
    F30,
    /// General-purpose function key.
    F31,
    /// General-purpose function key.
    F32,
    /// General-purpose function key.
    F33,
    /// General-purpose function key.
    F34,
    /// General-purpose function key.
    F35,

    /// An unknown key. Contains the OS native keycode/scancode
    Unknown(u32),
}

impl Key {
    /// Get this [Key] as a u64
    #[inline]
    pub const fn as_u64(self) -> u64 {
        let native_kc = if let Self::Unknown(native) = self {
            native
        } else {
            0
        };

        let variant = self.variant_index();

        ((variant as u64) << 32) | (native_kc as u64)
    }
}

impl core::hash::Hash for Key {
    #[inline(always)]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.as_u64());
    }
}

impl nohash_hasher::IsEnabled for Key {}

impl TryFrom<winit::keyboard::PhysicalKey> for Key {
    type Error = ();

    #[inline]
    #[allow(clippy::too_many_lines, reason = "Long match")]
    fn try_from(value: winit::keyboard::PhysicalKey) -> Result<Self, Self::Error> {
        match value {
            winit::keyboard::PhysicalKey::Unidentified(native_key_code) => {
                match winit_native_keycode_to_u32(native_key_code) {
                    Some(as_int) => Ok(Self::Unknown(as_int)),
                    None => {
                        log::warn!("Unidentified keycode, ignoring");
                        Err(())
                    }
                }
            }
            winit::keyboard::PhysicalKey::Code(kc) => match kc {
                winit::keyboard::KeyCode::Backquote => Ok(Self::Backquote),
                winit::keyboard::KeyCode::Backslash => Ok(Self::Backslash),
                winit::keyboard::KeyCode::BracketLeft => Ok(Self::BracketLeft),
                winit::keyboard::KeyCode::BracketRight => Ok(Self::BracketRight),
                winit::keyboard::KeyCode::Comma => Ok(Self::Comma),
                winit::keyboard::KeyCode::Digit0 => Ok(Self::Digit0),
                winit::keyboard::KeyCode::Digit1 => Ok(Self::Digit1),
                winit::keyboard::KeyCode::Digit2 => Ok(Self::Digit2),
                winit::keyboard::KeyCode::Digit3 => Ok(Self::Digit3),
                winit::keyboard::KeyCode::Digit4 => Ok(Self::Digit4),
                winit::keyboard::KeyCode::Digit5 => Ok(Self::Digit5),
                winit::keyboard::KeyCode::Digit6 => Ok(Self::Digit6),
                winit::keyboard::KeyCode::Digit7 => Ok(Self::Digit7),
                winit::keyboard::KeyCode::Digit8 => Ok(Self::Digit8),
                winit::keyboard::KeyCode::Digit9 => Ok(Self::Digit9),
                winit::keyboard::KeyCode::Equal => Ok(Self::Equal),
                winit::keyboard::KeyCode::IntlBackslash => Ok(Self::IntlBackslash),
                winit::keyboard::KeyCode::IntlRo => Ok(Self::IntlRo),
                winit::keyboard::KeyCode::IntlYen => Ok(Self::IntlYen),
                winit::keyboard::KeyCode::KeyA => Ok(Self::A),
                winit::keyboard::KeyCode::KeyB => Ok(Self::B),
                winit::keyboard::KeyCode::KeyC => Ok(Self::C),
                winit::keyboard::KeyCode::KeyD => Ok(Self::D),
                winit::keyboard::KeyCode::KeyE => Ok(Self::E),
                winit::keyboard::KeyCode::KeyF => Ok(Self::F),
                winit::keyboard::KeyCode::KeyG => Ok(Self::G),
                winit::keyboard::KeyCode::KeyH => Ok(Self::H),
                winit::keyboard::KeyCode::KeyI => Ok(Self::I),
                winit::keyboard::KeyCode::KeyJ => Ok(Self::J),
                winit::keyboard::KeyCode::KeyK => Ok(Self::K),
                winit::keyboard::KeyCode::KeyL => Ok(Self::L),
                winit::keyboard::KeyCode::KeyM => Ok(Self::M),
                winit::keyboard::KeyCode::KeyN => Ok(Self::N),
                winit::keyboard::KeyCode::KeyO => Ok(Self::O),
                winit::keyboard::KeyCode::KeyP => Ok(Self::P),
                winit::keyboard::KeyCode::KeyQ => Ok(Self::Q),
                winit::keyboard::KeyCode::KeyR => Ok(Self::R),
                winit::keyboard::KeyCode::KeyS => Ok(Self::S),
                winit::keyboard::KeyCode::KeyT => Ok(Self::T),
                winit::keyboard::KeyCode::KeyU => Ok(Self::U),
                winit::keyboard::KeyCode::KeyV => Ok(Self::V),
                winit::keyboard::KeyCode::KeyW => Ok(Self::W),
                winit::keyboard::KeyCode::KeyX => Ok(Self::X),
                winit::keyboard::KeyCode::KeyY => Ok(Self::Y),
                winit::keyboard::KeyCode::KeyZ => Ok(Self::Z),
                winit::keyboard::KeyCode::Minus => Ok(Self::Minus),
                winit::keyboard::KeyCode::Period => Ok(Self::Period),
                winit::keyboard::KeyCode::Quote => Ok(Self::Quote),
                winit::keyboard::KeyCode::Semicolon => Ok(Self::Semicolon),
                winit::keyboard::KeyCode::Slash => Ok(Self::Slash),
                winit::keyboard::KeyCode::AltLeft => Ok(Self::AltLeft),
                winit::keyboard::KeyCode::AltRight => Ok(Self::AltRight),
                winit::keyboard::KeyCode::Backspace => Ok(Self::Backspace),
                winit::keyboard::KeyCode::CapsLock => Ok(Self::CapsLock),
                winit::keyboard::KeyCode::ContextMenu => Ok(Self::ContextMenu),
                winit::keyboard::KeyCode::ControlLeft => Ok(Self::ControlLeft),
                winit::keyboard::KeyCode::ControlRight => Ok(Self::ControlRight),
                winit::keyboard::KeyCode::Enter => Ok(Self::Enter),
                winit::keyboard::KeyCode::SuperLeft => Ok(Self::SuperLeft),
                winit::keyboard::KeyCode::SuperRight => Ok(Self::SuperRight),
                winit::keyboard::KeyCode::ShiftLeft => Ok(Self::ShiftLeft),
                winit::keyboard::KeyCode::ShiftRight => Ok(Self::ShiftRight),
                winit::keyboard::KeyCode::Space => Ok(Self::Space),
                winit::keyboard::KeyCode::Tab => Ok(Self::Tab),
                winit::keyboard::KeyCode::Convert => Ok(Self::Convert),
                winit::keyboard::KeyCode::KanaMode => Ok(Self::KanaMode),
                winit::keyboard::KeyCode::Lang1 => Ok(Self::Lang1),
                winit::keyboard::KeyCode::Lang2 => Ok(Self::Lang2),
                winit::keyboard::KeyCode::Lang3 => Ok(Self::Lang3),
                winit::keyboard::KeyCode::Lang4 => Ok(Self::Lang4),
                winit::keyboard::KeyCode::Lang5 => Ok(Self::Lang5),
                winit::keyboard::KeyCode::NonConvert => Ok(Self::NonConvert),
                winit::keyboard::KeyCode::Delete => Ok(Self::Delete),
                winit::keyboard::KeyCode::End => Ok(Self::End),
                winit::keyboard::KeyCode::Help => Ok(Self::Help),
                winit::keyboard::KeyCode::Home => Ok(Self::Home),
                winit::keyboard::KeyCode::Insert => Ok(Self::Insert),
                winit::keyboard::KeyCode::PageDown => Ok(Self::PageDown),
                winit::keyboard::KeyCode::PageUp => Ok(Self::PageUp),
                winit::keyboard::KeyCode::ArrowDown => Ok(Self::ArrowDown),
                winit::keyboard::KeyCode::ArrowLeft => Ok(Self::ArrowLeft),
                winit::keyboard::KeyCode::ArrowRight => Ok(Self::ArrowRight),
                winit::keyboard::KeyCode::ArrowUp => Ok(Self::ArrowUp),
                winit::keyboard::KeyCode::NumLock => Ok(Self::NumLock),
                winit::keyboard::KeyCode::Numpad0 => Ok(Self::Numpad0),
                winit::keyboard::KeyCode::Numpad1 => Ok(Self::Numpad1),
                winit::keyboard::KeyCode::Numpad2 => Ok(Self::Numpad2),
                winit::keyboard::KeyCode::Numpad3 => Ok(Self::Numpad3),
                winit::keyboard::KeyCode::Numpad4 => Ok(Self::Numpad4),
                winit::keyboard::KeyCode::Numpad5 => Ok(Self::Numpad5),
                winit::keyboard::KeyCode::Numpad6 => Ok(Self::Numpad6),
                winit::keyboard::KeyCode::Numpad7 => Ok(Self::Numpad7),
                winit::keyboard::KeyCode::Numpad8 => Ok(Self::Numpad8),
                winit::keyboard::KeyCode::Numpad9 => Ok(Self::Numpad9),
                winit::keyboard::KeyCode::NumpadAdd => Ok(Self::NumpadAdd),
                winit::keyboard::KeyCode::NumpadBackspace => Ok(Self::NumpadBackspace),
                winit::keyboard::KeyCode::NumpadClear => Ok(Self::NumpadClear),
                winit::keyboard::KeyCode::NumpadClearEntry => Ok(Self::NumpadClearEntry),
                winit::keyboard::KeyCode::NumpadComma => Ok(Self::NumpadComma),
                winit::keyboard::KeyCode::NumpadDecimal => Ok(Self::NumpadDecimal),
                winit::keyboard::KeyCode::NumpadDivide => Ok(Self::NumpadDivide),
                winit::keyboard::KeyCode::NumpadEnter => Ok(Self::NumpadEnter),
                winit::keyboard::KeyCode::NumpadEqual => Ok(Self::NumpadEqual),
                winit::keyboard::KeyCode::NumpadHash => Ok(Self::NumpadHash),
                winit::keyboard::KeyCode::NumpadMemoryAdd => Ok(Self::NumpadMemoryAdd),
                winit::keyboard::KeyCode::NumpadMemoryClear => Ok(Self::NumpadMemoryClear),
                winit::keyboard::KeyCode::NumpadMemoryRecall => Ok(Self::NumpadMemoryRecall),
                winit::keyboard::KeyCode::NumpadMemoryStore => Ok(Self::NumpadMemoryStore),
                winit::keyboard::KeyCode::NumpadMemorySubtract => Ok(Self::NumpadMemorySubtract),
                winit::keyboard::KeyCode::NumpadMultiply => Ok(Self::NumpadMultiply),
                winit::keyboard::KeyCode::NumpadParenLeft => Ok(Self::NumpadParenLeft),
                winit::keyboard::KeyCode::NumpadParenRight => Ok(Self::NumpadParenRight),
                winit::keyboard::KeyCode::NumpadStar => Ok(Self::NumpadStar),
                winit::keyboard::KeyCode::NumpadSubtract => Ok(Self::NumpadSubtract),
                winit::keyboard::KeyCode::Escape => Ok(Self::Escape),
                winit::keyboard::KeyCode::Fn => Ok(Self::Fn),
                winit::keyboard::KeyCode::FnLock => Ok(Self::FnLock),
                winit::keyboard::KeyCode::PrintScreen => Ok(Self::PrintScreen),
                winit::keyboard::KeyCode::ScrollLock => Ok(Self::ScrollLock),
                winit::keyboard::KeyCode::Pause => Ok(Self::Pause),
                winit::keyboard::KeyCode::BrowserBack => Ok(Self::BrowserBack),
                winit::keyboard::KeyCode::BrowserFavorites => Ok(Self::BrowserFavorites),
                winit::keyboard::KeyCode::BrowserForward => Ok(Self::BrowserForward),
                winit::keyboard::KeyCode::BrowserHome => Ok(Self::BrowserHome),
                winit::keyboard::KeyCode::BrowserRefresh => Ok(Self::BrowserRefresh),
                winit::keyboard::KeyCode::BrowserSearch => Ok(Self::BrowserSearch),
                winit::keyboard::KeyCode::BrowserStop => Ok(Self::BrowserStop),
                winit::keyboard::KeyCode::Eject => Ok(Self::Eject),
                winit::keyboard::KeyCode::LaunchApp1 => Ok(Self::LaunchApp1),
                winit::keyboard::KeyCode::LaunchApp2 => Ok(Self::LaunchApp2),
                winit::keyboard::KeyCode::LaunchMail => Ok(Self::LaunchMail),
                winit::keyboard::KeyCode::MediaPlayPause => Ok(Self::MediaPlayPause),
                winit::keyboard::KeyCode::MediaSelect => Ok(Self::MediaSelect),
                winit::keyboard::KeyCode::MediaStop => Ok(Self::MediaStop),
                winit::keyboard::KeyCode::MediaTrackNext => Ok(Self::MediaTrackNext),
                winit::keyboard::KeyCode::MediaTrackPrevious => Ok(Self::MediaTrackPrevious),
                winit::keyboard::KeyCode::Power => Ok(Self::Power),
                winit::keyboard::KeyCode::Sleep => Ok(Self::Sleep),
                winit::keyboard::KeyCode::AudioVolumeDown => Ok(Self::AudioVolumeDown),
                winit::keyboard::KeyCode::AudioVolumeMute => Ok(Self::AudioVolumeMute),
                winit::keyboard::KeyCode::AudioVolumeUp => Ok(Self::AudioVolumeUp),
                winit::keyboard::KeyCode::WakeUp => Ok(Self::WakeUp),
                winit::keyboard::KeyCode::Meta => Ok(Self::Meta),
                winit::keyboard::KeyCode::Hyper => Ok(Self::Hyper),
                winit::keyboard::KeyCode::Turbo => Ok(Self::Turbo),
                winit::keyboard::KeyCode::Abort => Ok(Self::Abort),
                winit::keyboard::KeyCode::Resume => Ok(Self::Resume),
                winit::keyboard::KeyCode::Suspend => Ok(Self::Suspend),
                winit::keyboard::KeyCode::Again => Ok(Self::Again),
                winit::keyboard::KeyCode::Copy => Ok(Self::Copy),
                winit::keyboard::KeyCode::Cut => Ok(Self::Cut),
                winit::keyboard::KeyCode::Find => Ok(Self::Find),
                winit::keyboard::KeyCode::Open => Ok(Self::Open),
                winit::keyboard::KeyCode::Paste => Ok(Self::Paste),
                winit::keyboard::KeyCode::Props => Ok(Self::Props),
                winit::keyboard::KeyCode::Select => Ok(Self::Select),
                winit::keyboard::KeyCode::Undo => Ok(Self::Undo),
                winit::keyboard::KeyCode::Hiragana => Ok(Self::Hiragana),
                winit::keyboard::KeyCode::Katakana => Ok(Self::Katakana),
                winit::keyboard::KeyCode::F1 => Ok(Self::F1),
                winit::keyboard::KeyCode::F2 => Ok(Self::F2),
                winit::keyboard::KeyCode::F3 => Ok(Self::F3),
                winit::keyboard::KeyCode::F4 => Ok(Self::F4),
                winit::keyboard::KeyCode::F5 => Ok(Self::F5),
                winit::keyboard::KeyCode::F6 => Ok(Self::F6),
                winit::keyboard::KeyCode::F7 => Ok(Self::F7),
                winit::keyboard::KeyCode::F8 => Ok(Self::F8),
                winit::keyboard::KeyCode::F9 => Ok(Self::F9),
                winit::keyboard::KeyCode::F10 => Ok(Self::F10),
                winit::keyboard::KeyCode::F11 => Ok(Self::F11),
                winit::keyboard::KeyCode::F12 => Ok(Self::F12),
                winit::keyboard::KeyCode::F13 => Ok(Self::F13),
                winit::keyboard::KeyCode::F14 => Ok(Self::F14),
                winit::keyboard::KeyCode::F15 => Ok(Self::F15),
                winit::keyboard::KeyCode::F16 => Ok(Self::F16),
                winit::keyboard::KeyCode::F17 => Ok(Self::F17),
                winit::keyboard::KeyCode::F18 => Ok(Self::F18),
                winit::keyboard::KeyCode::F19 => Ok(Self::F19),
                winit::keyboard::KeyCode::F20 => Ok(Self::F20),
                winit::keyboard::KeyCode::F21 => Ok(Self::F21),
                winit::keyboard::KeyCode::F22 => Ok(Self::F22),
                winit::keyboard::KeyCode::F23 => Ok(Self::F23),
                winit::keyboard::KeyCode::F24 => Ok(Self::F24),
                winit::keyboard::KeyCode::F25 => Ok(Self::F25),
                winit::keyboard::KeyCode::F26 => Ok(Self::F26),
                winit::keyboard::KeyCode::F27 => Ok(Self::F27),
                winit::keyboard::KeyCode::F28 => Ok(Self::F28),
                winit::keyboard::KeyCode::F29 => Ok(Self::F29),
                winit::keyboard::KeyCode::F30 => Ok(Self::F30),
                winit::keyboard::KeyCode::F31 => Ok(Self::F31),
                winit::keyboard::KeyCode::F32 => Ok(Self::F32),
                winit::keyboard::KeyCode::F33 => Ok(Self::F33),
                winit::keyboard::KeyCode::F34 => Ok(Self::F34),
                winit::keyboard::KeyCode::F35 => Ok(Self::F35),
                other => {
                    log::warn!("Unknown keycode: {other:?}");
                    other.to_scancode().ok_or(()).map(Self::Unknown)
                }
            },
        }
    }
}
