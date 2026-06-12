//! Audio clip asset

use serde::Deserialize;
use serde::Serialize;

use crate::SerializedAsset;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedAudioClip {
    pub looped: bool,

    pub format: AudioClipFormat,

    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}

impl SerializedAsset for SerializedAudioClip {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioClipFormat {
    Wav,
    OggVorbis,
    Flac,
}
