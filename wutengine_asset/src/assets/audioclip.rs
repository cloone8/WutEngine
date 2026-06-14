//! Audio clip asset

use serde::Deserialize;
use serde::Serialize;

use crate::SerializedAsset;

/// A clip of audio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedAudioClip {
    /// The format of the clip
    pub format: AudioClipFormat,

    /// The raw data
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}

impl SerializedAsset for SerializedAudioClip {}

/// The format of a [SerializedAudioClip]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioClipFormat {
    /// WAV (.wav)
    Wav,

    /// OGG Vorbis (.ogg)
    OggVorbis,

    /// FLAC (.flac)
    Flac,

    /// MP3 (.mp3)
    Mp3,
}
