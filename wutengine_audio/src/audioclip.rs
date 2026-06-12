use core::convert::Infallible;

use wutengine_asset::Asset;
use wutengine_asset::assets::audioclip::AudioClipFormat;
use wutengine_asset::assets::audioclip::SerializedAudioClip;

#[derive(Debug, Clone)]
pub struct AudioClip {
    looped: bool,
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
            looped: serialized.looped,
            format: serialized.format,
            data: serialized.data.clone(),
        })
    }
}
