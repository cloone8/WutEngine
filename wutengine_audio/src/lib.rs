#![doc = include_str!("../README.md")]

use wutengine_util::InitOnce;

mod audioclip;
pub use audioclip::*;

static AUDIO_MANAGER: InitOnce<AudioManager> = InitOnce::new();

#[derive(Debug)]
struct AudioManager {
    sink: Option<rodio::MixerDeviceSink>,
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioManager {
    fn new() -> Self {
        let sink = Self::open_audio_device();

        Self { sink }
    }

    fn open_audio_device() -> Option<rodio::MixerDeviceSink> {
        profiling::function_scope!();

        let sink = match rodio::DeviceSinkBuilder::open_default_sink() {
            Ok(sink) => Some(sink),
            Err(e) => {
                log::error!(
                    "Failed to open default audio sink. No audio will be played. Error: {e}"
                );
                None
            }
        }?;

        log::debug!(
            "Opened default audio device with config: {:?}",
            sink.config()
        );

        Some(sink)
    }
}

/// Initializes the audio manager
#[doc(hidden)]
pub fn init() {
    profiling::function_scope!();

    InitOnce::init(&AUDIO_MANAGER, AudioManager::default());
}
