//! An audio clip

use core::convert::Infallible;

use wutengine_asset::Asset;
use wutengine_asset::assets::audioclip::AudioClipFormat;
use wutengine_asset::assets::audioclip::SerializedAudioClip;

/// A clip of audio
#[derive(Debug, Clone)]
pub struct AudioClip {
    format: AudioClipFormat,
    data: Vec<u8>,
}

impl Asset for AudioClip {
    type Serialized = SerializedAudioClip;

    type FromSerializedErr = Infallible;

    fn from_serialized(serialized: &Self::Serialized) -> Result<Self, Self::FromSerializedErr>
    where
        Self: Sized,
    {
        Ok(Self {
            format: serialized.format,
            data: serialized.data.clone(),
        })
    }
}
