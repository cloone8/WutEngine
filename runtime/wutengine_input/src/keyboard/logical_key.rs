use super::winit_nativekey_to_unknown_logical;

/// A logical keyboard input.
///
/// Used by non-location based input mappings, like UI and text input.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogicalKey {
    /// A character
    Character(char),

    /// A longer string of characters. Can be, for example, emoji
    String(String),

    /// A named key, like "shift" or "backspace"
    Named(LogicalNamed),

    /// An unknown key. Can be used for advanced input mappings
    Unknown(UnknownLogicalKey),
}

/// An unknown logical key with its identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnknownLogicalKey {
    /// Keycode
    Code(u32),

    /// String identifier
    String(String),
}

impl LogicalKey {
    /// Tries to map a [winit logical key](winit::keyboard::Key) to a [`LogicalKey`].
    /// Returns [`None`] if the key could not be mapped
    pub fn try_from_winit(logical: &winit::keyboard::Key) -> Option<Self> {
        match logical {
            winit::keyboard::Key::Named(named) => {
                let Some(logical_key) = LogicalNamed::from_winit(*named) else {
                    log::warn!("Unidentified logical key: {:?}", *named);
                    return None;
                };

                Some(Self::Named(logical_key))
            }
            winit::keyboard::Key::Character(ch) => {
                if ch.chars().take(2).count() > 1 {
                    Some(LogicalKey::String(ch.to_string()))
                } else {
                    Some(LogicalKey::Character(ch.chars().next().unwrap()))
                }
            }
            winit::keyboard::Key::Unidentified(native_key) => Some(Self::Unknown(
                winit_nativekey_to_unknown_logical(native_key.clone())?,
            )),
            winit::keyboard::Key::Dead(_) => None,
        }
    }
}

/// A named non-character keyboard value.
///
/// Taken from [`winit 0.30.13`](https://github.com/rust-windowing/winit/tree/v0.30.13),
/// and modified to suit WutEngine APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[expect(clippy::doc_markdown, reason = "Too many false positives")]
pub enum LogicalNamed {
    /// The `Alt` (Alternative) key.
    ///
    /// This key enables the alternate modifier function for interpreting concurrent or subsequent
    /// keyboard input. This key value is also used for the Apple <kbd>Option</kbd> key.
    Alt,
    /// The Alternate Graphics (<kbd>AltGr</kbd> or <kbd>AltGraph</kbd>) key.
    ///
    /// This key is used enable the ISO Level 3 shift modifier (the standard `Shift` key is the
    /// level 2 modifier).
    AltGraph,
    /// The `Caps Lock` (Capital) key.
    ///
    /// Toggle capital character lock function for interpreting subsequent keyboard input event.
    CapsLock,
    /// The `Control` or `Ctrl` key.
    ///
    /// Used to enable control modifier function for interpreting concurrent or subsequent keyboard
    /// input.
    Control,
    /// The Function switch `Fn` key. Activating this key simultaneously with another key changes
    /// that key’s value to an alternate character or function. This key is often handled directly
    /// in the keyboard hardware and does not usually generate key events.
    Fn,
    /// The Function-Lock (`FnLock` or `F-Lock`) key. Activating this key switches the mode of the
    /// keyboard to changes some keys' values to an alternate character or function. This key is
    /// often handled directly in the keyboard hardware and does not usually generate key events.
    FnLock,
    /// The `NumLock` or Number Lock key. Used to toggle numpad mode function for interpreting
    /// subsequent keyboard input.
    NumLock,
    /// Toggle between scrolling and cursor movement modes.
    ScrollLock,
    /// Used to enable shift modifier function for interpreting concurrent or subsequent keyboard
    /// input.
    Shift,
    /// The Symbol modifier key (used on some virtual keyboards).
    Symbol,

    /// Legacy key
    SymbolLock,

    /// Legacy modifier key. Also called "Super" in certain places.
    Meta,

    /// Legacy modifier key.
    Hyper,
    /// Used to enable "super" modifier function for interpreting concurrent or subsequent keyboard
    /// input. This key value is used for the "Windows Logo" key and the Apple `Command` or `⌘`
    /// key.
    ///
    /// Note: In some contexts (e.g. the Web) this is referred to as the "Meta" key.
    Super,
    /// The `Enter` or `↵` key. Used to activate current selection or accept current input. This
    /// key value is also used for the `Return` (Macintosh numpad) key. This key value is also
    /// used for the Android `KEYCODE_DPAD_CENTER`.
    Enter,
    /// The Horizontal Tabulation `Tab` key.
    Tab,
    /// Used in text to insert a space between words. Usually located below the character keys.
    Space,
    /// Navigate or traverse downward. (`KEYCODE_DPAD_DOWN`)
    ArrowDown,
    /// Navigate or traverse leftward. (`KEYCODE_DPAD_LEFT`)
    ArrowLeft,
    /// Navigate or traverse rightward. (`KEYCODE_DPAD_RIGHT`)
    ArrowRight,
    /// Navigate or traverse upward. (`KEYCODE_DPAD_UP`)
    ArrowUp,
    /// The End key, used with keyboard entry to go to the end of content (`KEYCODE_MOVE_END`).
    End,
    /// The Home key, used with keyboard entry, to go to start of content (`KEYCODE_MOVE_HOME`).
    /// For the mobile phone `Home` key (which goes to the phone’s main screen), use [``GoHome``].
    ///
    /// [``GoHome``]: Self::GoHome
    Home,
    /// Scroll down or display next page of content.
    PageDown,
    /// Scroll up or display previous page of content.
    PageUp,
    /// Used to remove the character to the left of the cursor. This key value is also used for
    /// the key labeled `Delete` on MacOS keyboards.
    Backspace,
    /// Remove the currently selected input.
    Clear,
    /// Copy the current selection. (`APPCOMMAND_COPY`)
    Copy,
    /// The Cursor Select key.
    CrSel,
    /// Cut the current selection. (`APPCOMMAND_CUT`)
    Cut,
    /// Used to delete the character to the right of the cursor. This key value is also used for
    /// the key labeled `Delete` on MacOS keyboards when `Fn` is active.
    Delete,
    /// The Erase to End of Field key. This key deletes all characters from the current cursor
    /// position to the end of the current field.
    EraseEof,
    /// The Extend Selection (Exsel) key.
    ExSel,
    /// Toggle between text modes for insertion or overtyping.
    /// (`KEYCODE_INSERT`)
    Insert,
    /// The Paste key. (`APPCOMMAND_PASTE`)
    Paste,
    /// Redo the last action. (`APPCOMMAND_REDO`)
    Redo,
    /// Undo the last action. (`APPCOMMAND_UNDO`)
    Undo,
    /// The Accept (Commit, OK) key. Accept current option or input method sequence conversion.
    Accept,
    /// Redo or repeat an action.
    Again,
    /// The Attention (Attn) key.
    Attn,

    /// The `cancel` key. Legacy
    Cancel,
    /// Show the application’s context menu.
    /// This key is commonly found between the right `Super` key and the right `Control` key.
    ContextMenu,
    /// The `Esc` key. This key was originally used to initiate an escape sequence, but is
    /// now more generally used to exit or "escape" the current context, such as closing a dialog
    /// or exiting full screen mode.
    Escape,

    /// The `execute` key
    Execute,
    /// Open the Find dialog. (`APPCOMMAND_FIND`)
    Find,
    /// Open a help dialog or toggle display of help information. (`APPCOMMAND_HELP`,
    /// `KEYCODE_HELP`)
    Help,
    /// Pause the current state or application (as appropriate).
    ///
    /// Note: Do not use this value for the `Pause` button on media controllers. Use `"MediaPause"`
    /// instead.
    Pause,
    /// Play or resume the current state or application (as appropriate).
    ///
    /// Note: Do not use this value for the `Play` button on media controllers. Use `"MediaPlay"`
    /// instead.
    Play,
    /// The properties (Props) key.
    Props,

    /// The `select` key
    Select,
    /// The ZoomIn key. (`KEYCODE_ZOOM_IN`)
    ZoomIn,
    /// The ZoomOut key. (`KEYCODE_ZOOM_OUT`)
    ZoomOut,
    /// The Brightness Down key. Typically controls the display brightness.
    /// (`KEYCODE_BRIGHTNESS_DOWN`)
    BrightnessDown,
    /// The Brightness Up key. Typically controls the display brightness. (`KEYCODE_BRIGHTNESS_UP`)
    BrightnessUp,
    /// Toggle removable media to eject (open) and insert (close) state. (`KEYCODE_MEDIA_EJECT`)
    Eject,

    /// The `log off` key
    LogOff,
    /// Toggle power state. (`KEYCODE_POWER`)
    /// Note: Note: Some devices might not expose this key to the operating environment.
    Power,
    /// The `PowerOff` key. Sometime called `PowerDown`.
    PowerOff,
    /// Initiate print-screen function.
    PrintScreen,
    /// The Hibernate key. This key saves the current state of the computer to disk so that it can
    /// be restored. The computer will then shutdown.
    Hibernate,
    /// The Standby key. This key turns off the display and places the computer into a low-power
    /// mode without completely shutting down. It is sometimes labelled `Suspend` or `Sleep` key.
    /// (`KEYCODE_SLEEP`)
    Standby,
    /// The WakeUp key. (`KEYCODE_WAKEUP`)
    WakeUp,
    /// Initiate the multi-candidate mode.
    AllCandidates,

    /// Alphanumeric key/mode
    Alphanumeric,
    /// Initiate the Code Input mode to allow characters to be entered by
    /// their code points.
    CodeInput,
    /// The Compose key, also known as "Multi_key" on the X Window System. This key acts in a
    /// manner similar to a dead key, triggering a mode where subsequent key presses are combined
    /// to produce a different character.
    Compose,
    /// Convert the current input method sequence.
    Convert,
    /// The Final Mode `Final` key used on some Asian keyboards, to enable the final mode for IMEs.
    FinalMode,
    /// Switch to the first character group. (ISO/IEC 9995)
    GroupFirst,
    /// Switch to the last character group. (ISO/IEC 9995)
    GroupLast,
    /// Switch to the next character group. (ISO/IEC 9995)
    GroupNext,
    /// Switch to the previous character group. (ISO/IEC 9995)
    GroupPrevious,
    /// Toggle between or cycle through input modes of IMEs.
    ModeChange,

    /// Next IME candidate
    NextCandidate,
    /// Accept current input method sequence without
    /// conversion in IMEs.
    NonConvert,

    /// Previous IME candidate
    PreviousCandidate,

    /// IME process
    Process,

    /// IME single candidate
    SingleCandidate,
    /// Toggle between Hangul and English modes.
    HangulMode,

    /// Toggle Hanja mode
    HanjaMode,

    /// Toggle Junja mode
    JunjaMode,
    /// The Eisu key. This key may close the IME, but its purpose is defined by the current IME.
    /// (`KEYCODE_EISU`)
    Eisu,
    /// The (Half-Width) Characters key.
    Hankaku,
    /// The Hiragana (Japanese Kana characters) key.
    Hiragana,
    /// The Hiragana/Katakana toggle key. (`KEYCODE_KATAKANA_HIRAGANA`)
    HiraganaKatakana,
    /// The Kana Mode (Kana Lock) key. This key is used to enter hiragana mode (typically from
    /// romaji mode).
    KanaMode,
    /// The Kanji (Japanese name for ideographic characters of Chinese origin) Mode key. This key
    /// is typically used to switch to a hiragana keyboard for the purpose of converting input
    /// into kanji. (`KEYCODE_KANA`)
    KanjiMode,
    /// The Katakana (Japanese Kana characters) key.
    Katakana,
    /// The Roman characters function key.
    Romaji,
    /// The Zenkaku (Full-Width) Characters key.
    Zenkaku,
    /// The Zenkaku/Hankaku (full-width/half-width) toggle key. (`KEYCODE_ZENKAKU_HANKAKU`)
    ZenkakuHankaku,
    /// General purpose virtual function key, as index 1.
    Soft1,
    /// General purpose virtual function key, as index 2.
    Soft2,
    /// General purpose virtual function key, as index 3.
    Soft3,
    /// General purpose virtual function key, as index 4.
    Soft4,
    /// Select next (numerically or logically) lower channel. (`APPCOMMAND_MEDIA_CHANNEL_DOWN`,
    /// `KEYCODE_CHANNEL_DOWN`)
    ChannelDown,
    /// Select next (numerically or logically) higher channel. (`APPCOMMAND_MEDIA_CHANNEL_UP`,
    /// `KEYCODE_CHANNEL_UP`)
    ChannelUp,
    /// Close the current document or message (Note: This doesn’t close the application).
    /// (`APPCOMMAND_CLOSE`)
    Close,
    /// Open an editor to forward the current message. (`APPCOMMAND_FORWARD_MAIL`)
    MailForward,
    /// Open an editor to reply to the current message. (`APPCOMMAND_REPLY_TO_MAIL`)
    MailReply,
    /// Send the current message. (`APPCOMMAND_SEND_MAIL`)
    MailSend,
    /// Close the current media, for example to close a CD or DVD tray. (`KEYCODE_MEDIA_CLOSE`)
    MediaClose,
    /// Initiate or continue forward playback at faster than normal speed, or increase speed if
    /// already fast forwarding. (`APPCOMMAND_MEDIA_FAST_FORWARD`, `KEYCODE_MEDIA_FAST_FORWARD`)
    MediaFastForward,
    /// Pause the currently playing media. (`APPCOMMAND_MEDIA_PAUSE`, `KEYCODE_MEDIA_PAUSE`)
    ///
    /// Note: Media controller devices should use this value rather than `"Pause"` for their pause
    /// keys.
    MediaPause,
    /// Initiate or continue media playback at normal speed, if not currently playing at normal
    /// speed. (`APPCOMMAND_MEDIA_PLAY`, `KEYCODE_MEDIA_PLAY`)
    MediaPlay,
    /// Toggle media between play and pause states. (`APPCOMMAND_MEDIA_PLAY_PAUSE`,
    /// `KEYCODE_MEDIA_PLAY_PAUSE`)
    MediaPlayPause,
    /// Initiate or resume recording of currently selected media. (`APPCOMMAND_MEDIA_RECORD`,
    /// `KEYCODE_MEDIA_RECORD`)
    MediaRecord,
    /// Initiate or continue reverse playback at faster than normal speed, or increase speed if
    /// already rewinding. (`APPCOMMAND_MEDIA_REWIND`, `KEYCODE_MEDIA_REWIND`)
    MediaRewind,
    /// Stop media playing, pausing, forwarding, rewinding, or recording, if not already stopped.
    /// (`APPCOMMAND_MEDIA_STOP`, `KEYCODE_MEDIA_STOP`)
    MediaStop,
    /// Seek to next media or program track. (`APPCOMMAND_MEDIA_NEXTTRACK`, `KEYCODE_MEDIA_NEXT`)
    MediaTrackNext,
    /// Seek to previous media or program track. (`APPCOMMAND_MEDIA_PREVIOUSTRACK`,
    /// `KEYCODE_MEDIA_PREVIOUS`)
    MediaTrackPrevious,
    /// Open a new document or message. (`APPCOMMAND_NEW`)
    New,
    /// Open an existing document or message. (`APPCOMMAND_OPEN`)
    Open,
    /// Print the current document or message. (`APPCOMMAND_PRINT`)
    Print,
    /// Save the current document or message. (`APPCOMMAND_SAVE`)
    Save,
    /// Spellcheck the current document or selection. (`APPCOMMAND_SPELL_CHECK`)
    SpellCheck,
    /// The `11` key found on media numpads that
    /// have buttons from `1` ... `12`.
    Key11,
    /// The `12` key found on media numpads that
    /// have buttons from `1` ... `12`.
    Key12,
    /// Adjust audio balance leftward. (`VK_AUDIO_BALANCE_LEFT`)
    AudioBalanceLeft,
    /// Adjust audio balance rightward. (`VK_AUDIO_BALANCE_RIGHT`)
    AudioBalanceRight,
    /// Decrease audio bass boost or cycle down through bass boost states. (`APPCOMMAND_BASS_DOWN`,
    /// `VK_BASS_BOOST_DOWN`)
    AudioBassBoostDown,
    /// Toggle bass boost on/off. (`APPCOMMAND_BASS_BOOST`)
    AudioBassBoostToggle,
    /// Increase audio bass boost or cycle up through bass boost states. (`APPCOMMAND_BASS_UP`,
    /// `VK_BASS_BOOST_UP`)
    AudioBassBoostUp,
    /// Adjust audio fader towards front. (`VK_FADER_FRONT`)
    AudioFaderFront,
    /// Adjust audio fader towards rear. (`VK_FADER_REAR`)
    AudioFaderRear,
    /// Advance surround audio mode to next available mode. (`VK_SURROUND_MODE_NEXT`)
    AudioSurroundModeNext,
    /// Decrease treble. (`APPCOMMAND_TREBLE_DOWN`)
    AudioTrebleDown,
    /// Increase treble. (`APPCOMMAND_TREBLE_UP`)
    AudioTrebleUp,
    /// Decrease audio volume. (`APPCOMMAND_VOLUME_DOWN`, `KEYCODE_VOLUME_DOWN`)
    AudioVolumeDown,
    /// Increase audio volume. (`APPCOMMAND_VOLUME_UP`, `KEYCODE_VOLUME_UP`)
    AudioVolumeUp,
    /// Toggle between muted state and prior volume level. (`APPCOMMAND_VOLUME_MUTE`,
    /// `KEYCODE_VOLUME_MUTE`)
    AudioVolumeMute,
    /// Toggle the microphone on/off. (`APPCOMMAND_MIC_ON_OFF_TOGGLE`)
    MicrophoneToggle,
    /// Decrease microphone volume. (`APPCOMMAND_MICROPHONE_VOLUME_DOWN`)
    MicrophoneVolumeDown,
    /// Increase microphone volume. (`APPCOMMAND_MICROPHONE_VOLUME_UP`)
    MicrophoneVolumeUp,
    /// Mute the microphone. (`APPCOMMAND_MICROPHONE_VOLUME_MUTE`, `KEYCODE_MUTE`)
    MicrophoneVolumeMute,
    /// Show correction list when a word is incorrectly identified. (`APPCOMMAND_CORRECTION_LIST`)
    SpeechCorrectionList,
    /// Toggle between dictation mode and command/control mode.
    /// (`APPCOMMAND_DICTATE_OR_COMMAND_CONTROL_TOGGLE`)
    SpeechInputToggle,
    /// The first generic "LaunchApplication" key. This is commonly associated with launching "My
    /// Computer", and may have a computer symbol on the key. (`APPCOMMAND_LAUNCH_APP1`)
    LaunchApplication1,
    /// The second generic "LaunchApplication" key. This is commonly associated with launching
    /// "Calculator", and may have a calculator symbol on the key. (`APPCOMMAND_LAUNCH_APP2`,
    /// `KEYCODE_CALCULATOR`)
    LaunchApplication2,
    /// The "Calendar" key. (`KEYCODE_CALENDAR`)
    LaunchCalendar,
    /// The "Contacts" key. (`KEYCODE_CONTACTS`)
    LaunchContacts,
    /// The "Mail" key. (`APPCOMMAND_LAUNCH_MAIL`)
    LaunchMail,
    /// The "Media Player" key. (`APPCOMMAND_LAUNCH_MEDIA_SELECT`)
    LaunchMediaPlayer,

    /// Launch music player key
    LaunchMusicPlayer,
    /// Launch phone key
    LaunchPhone,
    /// Launch screen saver key
    LaunchScreenSaver,
    /// Launch spreadsheet key
    LaunchSpreadsheet,
    /// Launch web browser key
    LaunchWebBrowser,
    /// Launch web cam key
    LaunchWebCam,
    /// Launch word processor key
    LaunchWordProcessor,
    /// Navigate to previous content or page in current history. (`APPCOMMAND_BROWSER_BACKWARD`)
    BrowserBack,
    /// Open the list of browser favorites. (`APPCOMMAND_BROWSER_FAVORITES`)
    BrowserFavorites,
    /// Navigate to next content or page in current history. (`APPCOMMAND_BROWSER_FORWARD`)
    BrowserForward,
    /// Go to the user’s preferred home page. (`APPCOMMAND_BROWSER_HOME`)
    BrowserHome,
    /// Refresh the current page or content. (`APPCOMMAND_BROWSER_REFRESH`)
    BrowserRefresh,
    /// Call up the user’s preferred search page. (`APPCOMMAND_BROWSER_SEARCH`)
    BrowserSearch,
    /// Stop loading the current page or content. (`APPCOMMAND_BROWSER_STOP`)
    BrowserStop,
    /// The Application switch key, which provides a list of recent apps to switch between.
    /// (`KEYCODE_APP_SWITCH`)
    AppSwitch,
    /// The Call key. (`KEYCODE_CALL`)
    Call,
    /// The Camera key. (`KEYCODE_CAMERA`)
    Camera,
    /// The Camera focus key. (`KEYCODE_FOCUS`)
    CameraFocus,
    /// The End Call key. (`KEYCODE_ENDCALL`)
    EndCall,
    /// The Back key. (`KEYCODE_BACK`)
    GoBack,
    /// The Home key, which goes to the phone’s main screen. (`KEYCODE_HOME`)
    GoHome,
    /// The Headset Hook key. (`KEYCODE_HEADSETHOOK`)
    HeadsetHook,

    /// Last number redial key
    LastNumberRedial,
    /// The Notification key. (`KEYCODE_NOTIFICATION`)
    Notification,
    /// Toggle between manner mode state: silent, vibrate, ring, ... (`KEYCODE_MANNER_MODE`)
    MannerMode,

    /// Voice dial key
    VoiceDial,
    /// Switch to viewing TV. (`KEYCODE_TV`)
    TV,
    /// TV 3D Mode. (`KEYCODE_3D_MODE`)
    TV3DMode,
    /// Toggle between antenna and cable input. (`KEYCODE_TV_ANTENNA_CABLE`)
    TVAntennaCable,
    /// Audio description. (`KEYCODE_TV_AUDIO_DESCRIPTION`)
    TVAudioDescription,
    /// Audio description mixing volume down. (`KEYCODE_TV_AUDIO_DESCRIPTION_MIX_DOWN`)
    TVAudioDescriptionMixDown,
    /// Audio description mixing volume up. (`KEYCODE_TV_AUDIO_DESCRIPTION_MIX_UP`)
    TVAudioDescriptionMixUp,
    /// Contents menu. (`KEYCODE_TV_CONTENTS_MENU`)
    TVContentsMenu,
    /// Contents menu. (`KEYCODE_TV_DATA_SERVICE`)
    TVDataService,
    /// Switch the input mode on an external TV. (`KEYCODE_TV_INPUT`)
    TVInput,
    /// Switch to component input #1. (`KEYCODE_TV_INPUT_COMPONENT_1`)
    TVInputComponent1,
    /// Switch to component input #2. (`KEYCODE_TV_INPUT_COMPONENT_2`)
    TVInputComponent2,
    /// Switch to composite input #1. (`KEYCODE_TV_INPUT_COMPOSITE_1`)
    TVInputComposite1,
    /// Switch to composite input #2. (`KEYCODE_TV_INPUT_COMPOSITE_2`)
    TVInputComposite2,
    /// Switch to HDMI input #1. (`KEYCODE_TV_INPUT_HDMI_1`)
    TVInputHDMI1,
    /// Switch to HDMI input #2. (`KEYCODE_TV_INPUT_HDMI_2`)
    TVInputHDMI2,
    /// Switch to HDMI input #3. (`KEYCODE_TV_INPUT_HDMI_3`)
    TVInputHDMI3,
    /// Switch to HDMI input #4. (`KEYCODE_TV_INPUT_HDMI_4`)
    TVInputHDMI4,
    /// Switch to VGA input #1. (`KEYCODE_TV_INPUT_VGA_1`)
    TVInputVGA1,
    /// Media context menu. (`KEYCODE_TV_MEDIA_CONTEXT_MENU`)
    TVMediaContext,
    /// Toggle network. (`KEYCODE_TV_NETWORK`)
    TVNetwork,
    /// Number entry. (`KEYCODE_TV_NUMBER_ENTRY`)
    TVNumberEntry,
    /// Toggle the power on an external TV. (`KEYCODE_TV_POWER`)
    TVPower,
    /// Radio. (`KEYCODE_TV_RADIO_SERVICE`)
    TVRadioService,
    /// Satellite. (`KEYCODE_TV_SATELLITE`)
    TVSatellite,
    /// Broadcast Satellite. (`KEYCODE_TV_SATELLITE_BS`)
    TVSatelliteBS,
    /// Communication Satellite. (`KEYCODE_TV_SATELLITE_CS`)
    TVSatelliteCS,
    /// Toggle between available satellites. (`KEYCODE_TV_SATELLITE_SERVICE`)
    TVSatelliteToggle,
    /// Analog Terrestrial. (`KEYCODE_TV_TERRESTRIAL_ANALOG`)
    TVTerrestrialAnalog,
    /// Digital Terrestrial. (`KEYCODE_TV_TERRESTRIAL_DIGITAL`)
    TVTerrestrialDigital,
    /// Timer programming. (`KEYCODE_TV_TIMER_PROGRAMMING`)
    TVTimer,
    /// Switch the input mode on an external AVR (audio/video receiver). (`KEYCODE_AVR_INPUT`)
    AVRInput,
    /// Toggle the power on an external AVR (audio/video receiver). (`KEYCODE_AVR_POWER`)
    AVRPower,
    /// General purpose color-coded media function key, as index 0 (red). (`VK_COLORED_KEY_0`,
    /// `KEYCODE_PROG_RED`)
    ColorF0Red,
    /// General purpose color-coded media function key, as index 1 (green). (`VK_COLORED_KEY_1`,
    /// `KEYCODE_PROG_GREEN`)
    ColorF1Green,
    /// General purpose color-coded media function key, as index 2 (yellow). (`VK_COLORED_KEY_2`,
    /// `KEYCODE_PROG_YELLOW`)
    ColorF2Yellow,
    /// General purpose color-coded media function key, as index 3 (blue). (`VK_COLORED_KEY_3`,
    /// `KEYCODE_PROG_BLUE`)
    ColorF3Blue,
    /// General purpose color-coded media function key, as index 4 (grey). (`VK_COLORED_KEY_4`)
    ColorF4Grey,
    /// General purpose color-coded media function key, as index 5 (brown). (`VK_COLORED_KEY_5`)
    ColorF5Brown,
    /// Toggle the display of Closed Captions. (`VK_CC`, `KEYCODE_CAPTIONS`)
    ClosedCaptionToggle,
    /// Adjust brightness of device, by toggling between or cycling through states. (`VK_DIMMER`)
    Dimmer,
    /// Swap video sources. (`VK_DISPLAY_SWAP`)
    DisplaySwap,
    /// Select Digital Video Recorder. (`KEYCODE_DVR`)
    Dvr,
    /// Exit the current application. (`VK_EXIT`)
    Exit,
    /// Clear program or content stored as favorite 0. (`VK_CLEAR_FAVORITE_0`)
    FavoriteClear0,
    /// Clear program or content stored as favorite 1. (`VK_CLEAR_FAVORITE_1`)
    FavoriteClear1,
    /// Clear program or content stored as favorite 2. (`VK_CLEAR_FAVORITE_2`)
    FavoriteClear2,
    /// Clear program or content stored as favorite 3. (`VK_CLEAR_FAVORITE_3`)
    FavoriteClear3,
    /// Select (recall) program or content stored as favorite 0. (`VK_RECALL_FAVORITE_0`)
    FavoriteRecall0,
    /// Select (recall) program or content stored as favorite 1. (`VK_RECALL_FAVORITE_1`)
    FavoriteRecall1,
    /// Select (recall) program or content stored as favorite 2. (`VK_RECALL_FAVORITE_2`)
    FavoriteRecall2,
    /// Select (recall) program or content stored as favorite 3. (`VK_RECALL_FAVORITE_3`)
    FavoriteRecall3,
    /// Store current program or content as favorite 0. (`VK_STORE_FAVORITE_0`)
    FavoriteStore0,
    /// Store current program or content as favorite 1. (`VK_STORE_FAVORITE_1`)
    FavoriteStore1,
    /// Store current program or content as favorite 2. (`VK_STORE_FAVORITE_2`)
    FavoriteStore2,
    /// Store current program or content as favorite 3. (`VK_STORE_FAVORITE_3`)
    FavoriteStore3,
    /// Toggle display of program or content guide. (`VK_GUIDE`, `KEYCODE_GUIDE`)
    Guide,
    /// If guide is active and displayed, then display next day’s content. (`VK_NEXT_DAY`)
    GuideNextDay,
    /// If guide is active and displayed, then display previous day’s content. (`VK_PREV_DAY`)
    GuidePreviousDay,
    /// Toggle display of information about currently selected context or media. (`VK_INFO`,
    /// `KEYCODE_INFO`)
    Info,
    /// Toggle instant replay. (`VK_INSTANT_REPLAY`)
    InstantReplay,
    /// Launch linked content, if available and appropriate. (`VK_LINK`)
    Link,
    /// List the current program. (`VK_LIST`)
    ListProgram,
    /// Toggle display listing of currently available live content or programs. (`VK_LIVE`)
    LiveContent,
    /// Lock or unlock current content or program. (`VK_LOCK`)
    Lock,
    /// Show a list of media applications: audio/video players and image viewers. (`VK_APPS`)
    ///
    /// Note: Do not confuse this key value with the Windows' `VK_APPS` / `VK_CONTEXT_MENU` key,
    /// which is encoded as `"ContextMenu"`.
    MediaApps,
    /// Audio track key. (`KEYCODE_MEDIA_AUDIO_TRACK`)
    MediaAudioTrack,
    /// Select previously selected channel or media. (`VK_LAST`, `KEYCODE_LAST_CHANNEL`)
    MediaLast,
    /// Skip backward to next content or program. (`KEYCODE_MEDIA_SKIP_BACKWARD`)
    MediaSkipBackward,
    /// Skip forward to next content or program. (`VK_SKIP`, `KEYCODE_MEDIA_SKIP_FORWARD`)
    MediaSkipForward,
    /// Step backward to next content or program. (`KEYCODE_MEDIA_STEP_BACKWARD`)
    MediaStepBackward,
    /// Step forward to next content or program. (`KEYCODE_MEDIA_STEP_FORWARD`)
    MediaStepForward,
    /// Media top menu. (`KEYCODE_MEDIA_TOP_MENU`)
    MediaTopMenu,
    /// Navigate in. (`KEYCODE_NAVIGATE_IN`)
    NavigateIn,
    /// Navigate to next key. (`KEYCODE_NAVIGATE_NEXT`)
    NavigateNext,
    /// Navigate out. (`KEYCODE_NAVIGATE_OUT`)
    NavigateOut,
    /// Navigate to previous key. (`KEYCODE_NAVIGATE_PREVIOUS`)
    NavigatePrevious,
    /// Cycle to next favorite channel (in favorites list). (`VK_NEXT_FAVORITE_CHANNEL`)
    NextFavoriteChannel,
    /// Cycle to next user profile (if there are multiple user profiles). (`VK_USER`)
    NextUserProfile,
    /// Access on-demand content or programs. (`VK_ON_DEMAND`)
    OnDemand,
    /// Pairing key to pair devices. (`KEYCODE_PAIRING`)
    Pairing,
    /// Move picture-in-picture window down. (`VK_PINP_DOWN`)
    PinPDown,
    /// Move picture-in-picture window. (`VK_PINP_MOVE`)
    PinPMove,
    /// Toggle display of picture-in-picture window. (`VK_PINP_TOGGLE`)
    PinPToggle,
    /// Move picture-in-picture window up. (`VK_PINP_UP`)
    PinPUp,
    /// Decrease media playback speed. (`VK_PLAY_SPEED_DOWN`)
    PlaySpeedDown,
    /// Reset playback to normal speed. (`VK_PLAY_SPEED_RESET`)
    PlaySpeedReset,
    /// Increase media playback speed. (`VK_PLAY_SPEED_UP`)
    PlaySpeedUp,
    /// Toggle random media or content shuffle mode. (`VK_RANDOM_TOGGLE`)
    RandomToggle,
    /// Not a physical key, but this key code is sent when the remote control battery is low.
    /// (`VK_RC_LOW_BATTERY`)
    RcLowBattery,
    /// Toggle or cycle between media recording speeds. (`VK_RECORD_SPEED_NEXT`)
    RecordSpeedNext,
    /// Toggle RF (radio frequency) input bypass mode (pass RF input directly to the RF output).
    /// (`VK_RF_BYPASS`)
    RfBypass,
    /// Toggle scan channels mode. (`VK_SCAN_CHANNELS_TOGGLE`)
    ScanChannelsToggle,
    /// Advance display screen mode to next available mode. (`VK_SCREEN_MODE_NEXT`)
    ScreenModeNext,
    /// Toggle display of device settings screen. (`VK_SETTINGS`, `KEYCODE_SETTINGS`)
    Settings,
    /// Toggle split screen mode. (`VK_SPLIT_SCREEN_TOGGLE`)
    SplitScreenToggle,
    /// Switch the input mode on an external STB (set top box). (`KEYCODE_STB_INPUT`)
    STBInput,
    /// Toggle the power on an external STB (set top box). (`KEYCODE_STB_POWER`)
    STBPower,
    /// Toggle display of subtitles, if available. (`VK_SUBTITLE`)
    Subtitle,
    /// Toggle display of teletext, if available (`VK_TELETEXT`, `KEYCODE_TV_TELETEXT`).
    Teletext,
    /// Advance video mode to next available mode. (`VK_VIDEO_MODE_NEXT`)
    VideoModeNext,
    /// Cause device to identify itself in some manner, e.g., audibly or visibly. (`VK_WINK`)
    Wink,
    /// Toggle between full-screen and scaled content, or alter magnification level. (`VK_ZOOM`,
    /// `KEYCODE_TV_ZOOM_MODE`)
    ZoomToggle,
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
}

impl LogicalNamed {
    /// Attempts to map a [winit named key](winit::keyboard::NamedKey) to a [`LogicalNamed`].
    /// If the key could not be mapped, returns [`None`]
    #[inline]
    #[expect(clippy::too_many_lines, reason = "Big match statement")]
    pub const fn from_winit(winit: winit::keyboard::NamedKey) -> Option<Self> {
        Some(match winit {
            winit::keyboard::NamedKey::Alt => Self::Alt,
            winit::keyboard::NamedKey::AltGraph => Self::AltGraph,
            winit::keyboard::NamedKey::CapsLock => Self::CapsLock,
            winit::keyboard::NamedKey::Control => Self::Control,
            winit::keyboard::NamedKey::Fn => Self::Fn,
            winit::keyboard::NamedKey::FnLock => Self::FnLock,
            winit::keyboard::NamedKey::NumLock => Self::NumLock,
            winit::keyboard::NamedKey::ScrollLock => Self::ScrollLock,
            winit::keyboard::NamedKey::Shift => Self::Shift,
            winit::keyboard::NamedKey::Symbol => Self::Symbol,
            winit::keyboard::NamedKey::SymbolLock => Self::SymbolLock,
            winit::keyboard::NamedKey::Meta => Self::Meta,
            winit::keyboard::NamedKey::Hyper => Self::Hyper,
            winit::keyboard::NamedKey::Super => Self::Super,
            winit::keyboard::NamedKey::Enter => Self::Enter,
            winit::keyboard::NamedKey::Tab => Self::Tab,
            winit::keyboard::NamedKey::Space => Self::Space,
            winit::keyboard::NamedKey::ArrowDown => Self::ArrowDown,
            winit::keyboard::NamedKey::ArrowLeft => Self::ArrowLeft,
            winit::keyboard::NamedKey::ArrowRight => Self::ArrowRight,
            winit::keyboard::NamedKey::ArrowUp => Self::ArrowUp,
            winit::keyboard::NamedKey::End => Self::End,
            winit::keyboard::NamedKey::Home => Self::Home,
            winit::keyboard::NamedKey::PageDown => Self::PageDown,
            winit::keyboard::NamedKey::PageUp => Self::PageUp,
            winit::keyboard::NamedKey::Backspace => Self::Backspace,
            winit::keyboard::NamedKey::Clear => Self::Clear,
            winit::keyboard::NamedKey::Copy => Self::Copy,
            winit::keyboard::NamedKey::CrSel => Self::CrSel,
            winit::keyboard::NamedKey::Cut => Self::Cut,
            winit::keyboard::NamedKey::Delete => Self::Delete,
            winit::keyboard::NamedKey::EraseEof => Self::EraseEof,
            winit::keyboard::NamedKey::ExSel => Self::ExSel,
            winit::keyboard::NamedKey::Insert => Self::Insert,
            winit::keyboard::NamedKey::Paste => Self::Paste,
            winit::keyboard::NamedKey::Redo => Self::Redo,
            winit::keyboard::NamedKey::Undo => Self::Undo,
            winit::keyboard::NamedKey::Accept => Self::Accept,
            winit::keyboard::NamedKey::Again => Self::Again,
            winit::keyboard::NamedKey::Attn => Self::Attn,
            winit::keyboard::NamedKey::Cancel => Self::Cancel,
            winit::keyboard::NamedKey::ContextMenu => Self::ContextMenu,
            winit::keyboard::NamedKey::Escape => Self::Escape,
            winit::keyboard::NamedKey::Execute => Self::Execute,
            winit::keyboard::NamedKey::Find => Self::Find,
            winit::keyboard::NamedKey::Help => Self::Help,
            winit::keyboard::NamedKey::Pause => Self::Pause,
            winit::keyboard::NamedKey::Play => Self::Play,
            winit::keyboard::NamedKey::Props => Self::Props,
            winit::keyboard::NamedKey::Select => Self::Select,
            winit::keyboard::NamedKey::ZoomIn => Self::ZoomIn,
            winit::keyboard::NamedKey::ZoomOut => Self::ZoomOut,
            winit::keyboard::NamedKey::BrightnessDown => Self::BrightnessDown,
            winit::keyboard::NamedKey::BrightnessUp => Self::BrightnessUp,
            winit::keyboard::NamedKey::Eject => Self::Eject,
            winit::keyboard::NamedKey::LogOff => Self::LogOff,
            winit::keyboard::NamedKey::Power => Self::Power,
            winit::keyboard::NamedKey::PowerOff => Self::PowerOff,
            winit::keyboard::NamedKey::PrintScreen => Self::PrintScreen,
            winit::keyboard::NamedKey::Hibernate => Self::Hibernate,
            winit::keyboard::NamedKey::Standby => Self::Standby,
            winit::keyboard::NamedKey::WakeUp => Self::WakeUp,
            winit::keyboard::NamedKey::AllCandidates => Self::AllCandidates,
            winit::keyboard::NamedKey::Alphanumeric => Self::Alphanumeric,
            winit::keyboard::NamedKey::CodeInput => Self::CodeInput,
            winit::keyboard::NamedKey::Compose => Self::Compose,
            winit::keyboard::NamedKey::Convert => Self::Convert,
            winit::keyboard::NamedKey::FinalMode => Self::FinalMode,
            winit::keyboard::NamedKey::GroupFirst => Self::GroupFirst,
            winit::keyboard::NamedKey::GroupLast => Self::GroupLast,
            winit::keyboard::NamedKey::GroupNext => Self::GroupNext,
            winit::keyboard::NamedKey::GroupPrevious => Self::GroupPrevious,
            winit::keyboard::NamedKey::ModeChange => Self::ModeChange,
            winit::keyboard::NamedKey::NextCandidate => Self::NextCandidate,
            winit::keyboard::NamedKey::NonConvert => Self::NonConvert,
            winit::keyboard::NamedKey::PreviousCandidate => Self::PreviousCandidate,
            winit::keyboard::NamedKey::Process => Self::Process,
            winit::keyboard::NamedKey::SingleCandidate => Self::SingleCandidate,
            winit::keyboard::NamedKey::HangulMode => Self::HangulMode,
            winit::keyboard::NamedKey::HanjaMode => Self::HanjaMode,
            winit::keyboard::NamedKey::JunjaMode => Self::JunjaMode,
            winit::keyboard::NamedKey::Eisu => Self::Eisu,
            winit::keyboard::NamedKey::Hankaku => Self::Hankaku,
            winit::keyboard::NamedKey::Hiragana => Self::Hiragana,
            winit::keyboard::NamedKey::HiraganaKatakana => Self::HiraganaKatakana,
            winit::keyboard::NamedKey::KanaMode => Self::KanaMode,
            winit::keyboard::NamedKey::KanjiMode => Self::KanjiMode,
            winit::keyboard::NamedKey::Katakana => Self::Katakana,
            winit::keyboard::NamedKey::Romaji => Self::Romaji,
            winit::keyboard::NamedKey::Zenkaku => Self::Zenkaku,
            winit::keyboard::NamedKey::ZenkakuHankaku => Self::ZenkakuHankaku,
            winit::keyboard::NamedKey::Soft1 => Self::Soft1,
            winit::keyboard::NamedKey::Soft2 => Self::Soft2,
            winit::keyboard::NamedKey::Soft3 => Self::Soft3,
            winit::keyboard::NamedKey::Soft4 => Self::Soft4,
            winit::keyboard::NamedKey::ChannelDown => Self::ChannelDown,
            winit::keyboard::NamedKey::ChannelUp => Self::ChannelUp,
            winit::keyboard::NamedKey::Close => Self::Close,
            winit::keyboard::NamedKey::MailForward => Self::MailForward,
            winit::keyboard::NamedKey::MailReply => Self::MailReply,
            winit::keyboard::NamedKey::MailSend => Self::MailSend,
            winit::keyboard::NamedKey::MediaClose => Self::MediaClose,
            winit::keyboard::NamedKey::MediaFastForward => Self::MediaFastForward,
            winit::keyboard::NamedKey::MediaPause => Self::MediaPause,
            winit::keyboard::NamedKey::MediaPlay => Self::MediaPlay,
            winit::keyboard::NamedKey::MediaPlayPause => Self::MediaPlayPause,
            winit::keyboard::NamedKey::MediaRecord => Self::MediaRecord,
            winit::keyboard::NamedKey::MediaRewind => Self::MediaRewind,
            winit::keyboard::NamedKey::MediaStop => Self::MediaStop,
            winit::keyboard::NamedKey::MediaTrackNext => Self::MediaTrackNext,
            winit::keyboard::NamedKey::MediaTrackPrevious => Self::MediaTrackPrevious,
            winit::keyboard::NamedKey::New => Self::New,
            winit::keyboard::NamedKey::Open => Self::Open,
            winit::keyboard::NamedKey::Print => Self::Print,
            winit::keyboard::NamedKey::Save => Self::Save,
            winit::keyboard::NamedKey::SpellCheck => Self::SpellCheck,
            winit::keyboard::NamedKey::Key11 => Self::Key11,
            winit::keyboard::NamedKey::Key12 => Self::Key12,
            winit::keyboard::NamedKey::AudioBalanceLeft => Self::AudioBalanceLeft,
            winit::keyboard::NamedKey::AudioBalanceRight => Self::AudioBalanceRight,
            winit::keyboard::NamedKey::AudioBassBoostDown => Self::AudioBassBoostDown,
            winit::keyboard::NamedKey::AudioBassBoostToggle => Self::AudioBassBoostToggle,
            winit::keyboard::NamedKey::AudioBassBoostUp => Self::AudioBassBoostUp,
            winit::keyboard::NamedKey::AudioFaderFront => Self::AudioFaderFront,
            winit::keyboard::NamedKey::AudioFaderRear => Self::AudioFaderRear,
            winit::keyboard::NamedKey::AudioSurroundModeNext => Self::AudioSurroundModeNext,
            winit::keyboard::NamedKey::AudioTrebleDown => Self::AudioTrebleDown,
            winit::keyboard::NamedKey::AudioTrebleUp => Self::AudioTrebleUp,
            winit::keyboard::NamedKey::AudioVolumeDown => Self::AudioVolumeDown,
            winit::keyboard::NamedKey::AudioVolumeUp => Self::AudioVolumeUp,
            winit::keyboard::NamedKey::AudioVolumeMute => Self::AudioVolumeMute,
            winit::keyboard::NamedKey::MicrophoneToggle => Self::MicrophoneToggle,
            winit::keyboard::NamedKey::MicrophoneVolumeDown => Self::MicrophoneVolumeDown,
            winit::keyboard::NamedKey::MicrophoneVolumeUp => Self::MicrophoneVolumeUp,
            winit::keyboard::NamedKey::MicrophoneVolumeMute => Self::MicrophoneVolumeMute,
            winit::keyboard::NamedKey::SpeechCorrectionList => Self::SpeechCorrectionList,
            winit::keyboard::NamedKey::SpeechInputToggle => Self::SpeechInputToggle,
            winit::keyboard::NamedKey::LaunchApplication1 => Self::LaunchApplication1,
            winit::keyboard::NamedKey::LaunchApplication2 => Self::LaunchApplication2,
            winit::keyboard::NamedKey::LaunchCalendar => Self::LaunchCalendar,
            winit::keyboard::NamedKey::LaunchContacts => Self::LaunchContacts,
            winit::keyboard::NamedKey::LaunchMail => Self::LaunchMail,
            winit::keyboard::NamedKey::LaunchMediaPlayer => Self::LaunchMediaPlayer,
            winit::keyboard::NamedKey::LaunchMusicPlayer => Self::LaunchMusicPlayer,
            winit::keyboard::NamedKey::LaunchPhone => Self::LaunchPhone,
            winit::keyboard::NamedKey::LaunchScreenSaver => Self::LaunchScreenSaver,
            winit::keyboard::NamedKey::LaunchSpreadsheet => Self::LaunchSpreadsheet,
            winit::keyboard::NamedKey::LaunchWebBrowser => Self::LaunchWebBrowser,
            winit::keyboard::NamedKey::LaunchWebCam => Self::LaunchWebCam,
            winit::keyboard::NamedKey::LaunchWordProcessor => Self::LaunchWordProcessor,
            winit::keyboard::NamedKey::BrowserBack => Self::BrowserBack,
            winit::keyboard::NamedKey::BrowserFavorites => Self::BrowserFavorites,
            winit::keyboard::NamedKey::BrowserForward => Self::BrowserForward,
            winit::keyboard::NamedKey::BrowserHome => Self::BrowserHome,
            winit::keyboard::NamedKey::BrowserRefresh => Self::BrowserRefresh,
            winit::keyboard::NamedKey::BrowserSearch => Self::BrowserSearch,
            winit::keyboard::NamedKey::BrowserStop => Self::BrowserStop,
            winit::keyboard::NamedKey::AppSwitch => Self::AppSwitch,
            winit::keyboard::NamedKey::Call => Self::Call,
            winit::keyboard::NamedKey::Camera => Self::Camera,
            winit::keyboard::NamedKey::CameraFocus => Self::CameraFocus,
            winit::keyboard::NamedKey::EndCall => Self::EndCall,
            winit::keyboard::NamedKey::GoBack => Self::GoBack,
            winit::keyboard::NamedKey::GoHome => Self::GoHome,
            winit::keyboard::NamedKey::HeadsetHook => Self::HeadsetHook,
            winit::keyboard::NamedKey::LastNumberRedial => Self::LastNumberRedial,
            winit::keyboard::NamedKey::Notification => Self::Notification,
            winit::keyboard::NamedKey::MannerMode => Self::MannerMode,
            winit::keyboard::NamedKey::VoiceDial => Self::VoiceDial,
            winit::keyboard::NamedKey::TV => Self::TV,
            winit::keyboard::NamedKey::TV3DMode => Self::TV3DMode,
            winit::keyboard::NamedKey::TVAntennaCable => Self::TVAntennaCable,
            winit::keyboard::NamedKey::TVAudioDescription => Self::TVAudioDescription,
            winit::keyboard::NamedKey::TVAudioDescriptionMixDown => Self::TVAudioDescriptionMixDown,
            winit::keyboard::NamedKey::TVAudioDescriptionMixUp => Self::TVAudioDescriptionMixUp,
            winit::keyboard::NamedKey::TVContentsMenu => Self::TVContentsMenu,
            winit::keyboard::NamedKey::TVDataService => Self::TVDataService,
            winit::keyboard::NamedKey::TVInput => Self::TVInput,
            winit::keyboard::NamedKey::TVInputComponent1 => Self::TVInputComponent1,
            winit::keyboard::NamedKey::TVInputComponent2 => Self::TVInputComponent2,
            winit::keyboard::NamedKey::TVInputComposite1 => Self::TVInputComposite1,
            winit::keyboard::NamedKey::TVInputComposite2 => Self::TVInputComposite2,
            winit::keyboard::NamedKey::TVInputHDMI1 => Self::TVInputHDMI1,
            winit::keyboard::NamedKey::TVInputHDMI2 => Self::TVInputHDMI2,
            winit::keyboard::NamedKey::TVInputHDMI3 => Self::TVInputHDMI3,
            winit::keyboard::NamedKey::TVInputHDMI4 => Self::TVInputHDMI4,
            winit::keyboard::NamedKey::TVInputVGA1 => Self::TVInputVGA1,
            winit::keyboard::NamedKey::TVMediaContext => Self::TVMediaContext,
            winit::keyboard::NamedKey::TVNetwork => Self::TVNetwork,
            winit::keyboard::NamedKey::TVNumberEntry => Self::TVNumberEntry,
            winit::keyboard::NamedKey::TVPower => Self::TVPower,
            winit::keyboard::NamedKey::TVRadioService => Self::TVRadioService,
            winit::keyboard::NamedKey::TVSatellite => Self::TVSatellite,
            winit::keyboard::NamedKey::TVSatelliteBS => Self::TVSatelliteBS,
            winit::keyboard::NamedKey::TVSatelliteCS => Self::TVSatelliteCS,
            winit::keyboard::NamedKey::TVSatelliteToggle => Self::TVSatelliteToggle,
            winit::keyboard::NamedKey::TVTerrestrialAnalog => Self::TVTerrestrialAnalog,
            winit::keyboard::NamedKey::TVTerrestrialDigital => Self::TVTerrestrialDigital,
            winit::keyboard::NamedKey::TVTimer => Self::TVTimer,
            winit::keyboard::NamedKey::AVRInput => Self::AVRInput,
            winit::keyboard::NamedKey::AVRPower => Self::AVRPower,
            winit::keyboard::NamedKey::ColorF0Red => Self::ColorF0Red,
            winit::keyboard::NamedKey::ColorF1Green => Self::ColorF1Green,
            winit::keyboard::NamedKey::ColorF2Yellow => Self::ColorF2Yellow,
            winit::keyboard::NamedKey::ColorF3Blue => Self::ColorF3Blue,
            winit::keyboard::NamedKey::ColorF4Grey => Self::ColorF4Grey,
            winit::keyboard::NamedKey::ColorF5Brown => Self::ColorF5Brown,
            winit::keyboard::NamedKey::ClosedCaptionToggle => Self::ClosedCaptionToggle,
            winit::keyboard::NamedKey::Dimmer => Self::Dimmer,
            winit::keyboard::NamedKey::DisplaySwap => Self::DisplaySwap,
            winit::keyboard::NamedKey::DVR => Self::Dvr,
            winit::keyboard::NamedKey::Exit => Self::Exit,
            winit::keyboard::NamedKey::FavoriteClear0 => Self::FavoriteClear0,
            winit::keyboard::NamedKey::FavoriteClear1 => Self::FavoriteClear1,
            winit::keyboard::NamedKey::FavoriteClear2 => Self::FavoriteClear2,
            winit::keyboard::NamedKey::FavoriteClear3 => Self::FavoriteClear3,
            winit::keyboard::NamedKey::FavoriteRecall0 => Self::FavoriteRecall0,
            winit::keyboard::NamedKey::FavoriteRecall1 => Self::FavoriteRecall1,
            winit::keyboard::NamedKey::FavoriteRecall2 => Self::FavoriteRecall2,
            winit::keyboard::NamedKey::FavoriteRecall3 => Self::FavoriteRecall3,
            winit::keyboard::NamedKey::FavoriteStore0 => Self::FavoriteStore0,
            winit::keyboard::NamedKey::FavoriteStore1 => Self::FavoriteStore1,
            winit::keyboard::NamedKey::FavoriteStore2 => Self::FavoriteStore2,
            winit::keyboard::NamedKey::FavoriteStore3 => Self::FavoriteStore3,
            winit::keyboard::NamedKey::Guide => Self::Guide,
            winit::keyboard::NamedKey::GuideNextDay => Self::GuideNextDay,
            winit::keyboard::NamedKey::GuidePreviousDay => Self::GuidePreviousDay,
            winit::keyboard::NamedKey::Info => Self::Info,
            winit::keyboard::NamedKey::InstantReplay => Self::InstantReplay,
            winit::keyboard::NamedKey::Link => Self::Link,
            winit::keyboard::NamedKey::ListProgram => Self::ListProgram,
            winit::keyboard::NamedKey::LiveContent => Self::LiveContent,
            winit::keyboard::NamedKey::Lock => Self::Lock,
            winit::keyboard::NamedKey::MediaApps => Self::MediaApps,
            winit::keyboard::NamedKey::MediaAudioTrack => Self::MediaAudioTrack,
            winit::keyboard::NamedKey::MediaLast => Self::MediaLast,
            winit::keyboard::NamedKey::MediaSkipBackward => Self::MediaSkipBackward,
            winit::keyboard::NamedKey::MediaSkipForward => Self::MediaSkipForward,
            winit::keyboard::NamedKey::MediaStepBackward => Self::MediaStepBackward,
            winit::keyboard::NamedKey::MediaStepForward => Self::MediaStepForward,
            winit::keyboard::NamedKey::MediaTopMenu => Self::MediaTopMenu,
            winit::keyboard::NamedKey::NavigateIn => Self::NavigateIn,
            winit::keyboard::NamedKey::NavigateNext => Self::NavigateNext,
            winit::keyboard::NamedKey::NavigateOut => Self::NavigateOut,
            winit::keyboard::NamedKey::NavigatePrevious => Self::NavigatePrevious,
            winit::keyboard::NamedKey::NextFavoriteChannel => Self::NextFavoriteChannel,
            winit::keyboard::NamedKey::NextUserProfile => Self::NextUserProfile,
            winit::keyboard::NamedKey::OnDemand => Self::OnDemand,
            winit::keyboard::NamedKey::Pairing => Self::Pairing,
            winit::keyboard::NamedKey::PinPDown => Self::PinPDown,
            winit::keyboard::NamedKey::PinPMove => Self::PinPMove,
            winit::keyboard::NamedKey::PinPToggle => Self::PinPToggle,
            winit::keyboard::NamedKey::PinPUp => Self::PinPUp,
            winit::keyboard::NamedKey::PlaySpeedDown => Self::PlaySpeedDown,
            winit::keyboard::NamedKey::PlaySpeedReset => Self::PlaySpeedReset,
            winit::keyboard::NamedKey::PlaySpeedUp => Self::PlaySpeedUp,
            winit::keyboard::NamedKey::RandomToggle => Self::RandomToggle,
            winit::keyboard::NamedKey::RcLowBattery => Self::RcLowBattery,
            winit::keyboard::NamedKey::RecordSpeedNext => Self::RecordSpeedNext,
            winit::keyboard::NamedKey::RfBypass => Self::RfBypass,
            winit::keyboard::NamedKey::ScanChannelsToggle => Self::ScanChannelsToggle,
            winit::keyboard::NamedKey::ScreenModeNext => Self::ScreenModeNext,
            winit::keyboard::NamedKey::Settings => Self::Settings,
            winit::keyboard::NamedKey::SplitScreenToggle => Self::SplitScreenToggle,
            winit::keyboard::NamedKey::STBInput => Self::STBInput,
            winit::keyboard::NamedKey::STBPower => Self::STBPower,
            winit::keyboard::NamedKey::Subtitle => Self::Subtitle,
            winit::keyboard::NamedKey::Teletext => Self::Teletext,
            winit::keyboard::NamedKey::VideoModeNext => Self::VideoModeNext,
            winit::keyboard::NamedKey::Wink => Self::Wink,
            winit::keyboard::NamedKey::ZoomToggle => Self::ZoomToggle,
            winit::keyboard::NamedKey::F1 => Self::F1,
            winit::keyboard::NamedKey::F2 => Self::F2,
            winit::keyboard::NamedKey::F3 => Self::F3,
            winit::keyboard::NamedKey::F4 => Self::F4,
            winit::keyboard::NamedKey::F5 => Self::F5,
            winit::keyboard::NamedKey::F6 => Self::F6,
            winit::keyboard::NamedKey::F7 => Self::F7,
            winit::keyboard::NamedKey::F8 => Self::F8,
            winit::keyboard::NamedKey::F9 => Self::F9,
            winit::keyboard::NamedKey::F10 => Self::F10,
            winit::keyboard::NamedKey::F11 => Self::F11,
            winit::keyboard::NamedKey::F12 => Self::F12,
            winit::keyboard::NamedKey::F13 => Self::F13,
            winit::keyboard::NamedKey::F14 => Self::F14,
            winit::keyboard::NamedKey::F15 => Self::F15,
            winit::keyboard::NamedKey::F16 => Self::F16,
            winit::keyboard::NamedKey::F17 => Self::F17,
            winit::keyboard::NamedKey::F18 => Self::F18,
            winit::keyboard::NamedKey::F19 => Self::F19,
            winit::keyboard::NamedKey::F20 => Self::F20,
            winit::keyboard::NamedKey::F21 => Self::F21,
            winit::keyboard::NamedKey::F22 => Self::F22,
            winit::keyboard::NamedKey::F23 => Self::F23,
            winit::keyboard::NamedKey::F24 => Self::F24,
            winit::keyboard::NamedKey::F25 => Self::F25,
            winit::keyboard::NamedKey::F26 => Self::F26,
            winit::keyboard::NamedKey::F27 => Self::F27,
            winit::keyboard::NamedKey::F28 => Self::F28,
            winit::keyboard::NamedKey::F29 => Self::F29,
            winit::keyboard::NamedKey::F30 => Self::F30,
            winit::keyboard::NamedKey::F31 => Self::F31,
            winit::keyboard::NamedKey::F32 => Self::F32,
            winit::keyboard::NamedKey::F33 => Self::F33,
            winit::keyboard::NamedKey::F34 => Self::F34,
            winit::keyboard::NamedKey::F35 => Self::F35,
            _ => {
                return None;
            }
        })
    }
}
