use wutengine_asset::AssetHandle;
use wutengine_audio::AudioClip;
use wutengine_audio::rodio::Source;

use crate::component::Component;

/// A component that plays audio
#[derive(Default)]
pub struct AudioPlayer {
    clip: Option<AssetHandle<AudioClip>>,
    player: Option<wutengine_audio::rodio::Player>,
    player_init: bool,
    clip_init: bool,
}

/// Public API
impl AudioPlayer {
    pub fn play(&mut self) {
        self.ensure_init();

        if let Some(player) = self.player.as_ref() {
            player.play();
        }
    }

    pub fn pause(&mut self) {
        self.ensure_init();

        if let Some(player) = self.player.as_ref() {
            player.pause();
        }
    }

    pub fn set_clip(&mut self, clip: Option<AssetHandle<AudioClip>>) {
        self.clip = clip;
        self.clip_init = false;

        let Some(player) = self.player.as_ref() else {
            return;
        };

        player.stop();
    }

    pub fn reset(&mut self) {
        self.ensure_init();

        let Some(player) = self.player.as_ref() else {
            return;
        };

        player.stop();
        self.clip_init = false;
        self.add_clip();
    }
}

/// Private API
impl AudioPlayer {
    fn ensure_init(&mut self) {
        self.init_player();
        self.add_clip();
    }

    fn init_player(&mut self) {
        if self.player_init {
            return;
        }

        self.player = wutengine_audio::new_player();
        self.player_init = true;
    }

    fn add_clip(&mut self) {
        assert!(self.player_init, "Player must be initialized");

        if self.clip_init {
            return;
        }

        self.clip_init = true;

        let (Some(player), Some(clip_asset)) = (self.player.as_ref(), self.clip.as_ref()) else {
            return;
        };

        let Some(clip) = clip_asset.get_ref() else {
            log::error!("Failed to load audio clip asset");
            return;
        };

        let decoder = match clip.new_decoder() {
            Ok(decoder) => decoder,
            Err(e) => {
                log::error!("Failed to get audio decoder for clip: {e}");
                return;
            }
        };

        player.append(decoder);
    }
}

impl Component for AudioPlayer {
    fn insert_default_component_systems(_manifest: &mut crate::runtime::SystemManifest)
    where
        Self: Sized,
    {
    }
}
