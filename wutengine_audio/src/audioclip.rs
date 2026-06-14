//! An audio clip

extern crate alloc;

use alloc::sync::Arc;
use core::convert::Infallible;

use wutengine_asset::Asset;
use wutengine_asset::assets::audioclip::AudioClipFormat;
use wutengine_asset::assets::audioclip::SerializedAudioClip;

/// A clip of audio
#[derive(Debug, Clone)]
pub struct AudioClip {
    format: AudioClipFormat,
    data: Arc<[u8]>,
}

impl AudioClip {
    pub fn new_decoder(
        &self,
    ) -> Result<rodio::Decoder<std::io::Cursor<Arc<[u8]>>>, rodio::decoder::DecoderError> {
        let source = std::io::Cursor::new(self.data.clone());

        match self.format {
            AudioClipFormat::Wav => rodio::Decoder::new_wav(source),
            AudioClipFormat::OggVorbis => rodio::Decoder::new_vorbis(source),
            AudioClipFormat::Flac => rodio::Decoder::new_flac(source),
            AudioClipFormat::Mp3 => rodio::Decoder::new_mp3(source),
        }
    }
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
            data: Arc::from(serialized.data.clone()),
        })
    }
}
