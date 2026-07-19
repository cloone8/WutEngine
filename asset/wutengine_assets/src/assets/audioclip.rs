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

impl SerializedAsset for SerializedAudioClip {
    const ID: uuid::NonNilUuid =
        uuid::NonNilUuid::new(uuid::uuid!("d408e523-0772-48dc-a77a-ed3d11c72ca0")).unwrap();
}

/// The format of a [`SerializedAudioClip`]
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
