//! An audio clip

extern crate alloc;

use alloc::sync::Arc;
use core::convert::Infallible;
use wutengine_assets::FromSerializedAsset;

use wutengine_assets::assets::audioclip::AudioClipFormat;
use wutengine_assets::assets::audioclip::SerializedAudioClip;

/// A clip of audio
#[derive(Debug, Clone)]
pub struct AudioClip {
    format: AudioClipFormat,
    data: Arc<[u8]>,
}

impl AudioClip {
    /// Creates a new decoder that stops after the clip is done. For a looping decoder, see [`Self::new_looped_decoder`]
    pub fn new_decoder(
        &self,
    ) -> Result<rodio::Decoder<std::io::Cursor<Arc<[u8]>>>, rodio::decoder::DecoderError> {
        self.new_builder().build()
    }

    /// Creates a new decoder that loops back to the start after the clip is done. For a non-looping decoder, see [`Self::new_decoder`]
    pub fn new_looped_decoder(
        &self,
    ) -> Result<
        rodio::decoder::LoopedDecoder<std::io::Cursor<Arc<[u8]>>>,
        rodio::decoder::DecoderError,
    > {
        self.new_builder().build_looped()
    }

    fn new_builder(&self) -> rodio::decoder::DecoderBuilder<std::io::Cursor<Arc<[u8]>>> {
        let source = std::io::Cursor::new(self.data.clone());

        rodio::Decoder::builder()
            .with_data(source)
            .with_byte_len(self.data.len() as u64)
            .with_gapless(true)
            .with_seekable(true)
            .with_hint(Self::format_to_hint(self.format))
    }

    const fn format_to_hint(format: AudioClipFormat) -> &'static str {
        match format {
            AudioClipFormat::Wav => "wav",
            AudioClipFormat::OggVorbis => "ogg",
            AudioClipFormat::Flac => "flac",
            AudioClipFormat::Mp3 => "mp3",
        }
    }
}

impl FromSerializedAsset for AudioClip {
    type Error = Infallible;

    type Serialized = SerializedAudioClip;

    fn from_serialized_asset(serialized: Self::Serialized) -> Result<Self, Self::Error> {
        Ok(Self {
            format: serialized.format,
            data: Arc::from(serialized.data),
        })
    }
}

// impl Asset for AudioClip {
//     type Serialized = SerializedAudioClip;

//     type FromSerializedErr = Infallible;

//     fn from_serialized(serialized: &Self::Serialized) -> Result<Self, Self::FromSerializedErr>
//     where
//         Self: Sized,
//     {
//         Ok(Self {
//             format: serialized.format,
//             data: Arc::from(serialized.data.clone()),
//         })
//     }
// }
