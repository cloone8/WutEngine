//! Frame pacer

use core::time::Duration;
use std::time::Instant;

use wutengine_util::assert_main_thread;

/// Frame pacer. Allows for pacing the amount of rendered frames
/// without vsync
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct FramePacer {
    frame_interval: Option<Duration>,
    prev_frame: Option<Instant>,
}

impl FramePacer {
    /// Sets the frame interval to a new value
    pub(crate) fn set_frame_interval(&mut self, frame_interval: Option<Duration>) {
        self.frame_interval = frame_interval;
    }

    /// Call this when a frame was rendered. Makes sure the time of the next frame is calculated properly
    pub(crate) fn frame_rendered(&mut self) {
        self.prev_frame = Some(Instant::now());
    }

    /// Blocks the calling thread until the time when the next frame should be rendered
    pub(crate) fn wait_for_limit(&self) {
        assert_main_thread!();

        //TODO: This is currently not accurate enough, it seems like the actual resulting FPS is
        // always lower (so the limiter sleeps for too long). Probably have to take into account the actual GPU
        // rendering time or something.

        let Some(prev_frame) = self.prev_frame else {
            // Nothing has been rendered yet
            return;
        };

        let Some(interval) = self.frame_interval else {
            // No limit
            return;
        };

        profiling::function_scope!();

        let next_frame_moment = prev_frame.checked_add(interval).unwrap();

        if Instant::now() >= next_frame_moment {
            // No sleep needed, continue
            return;
        }

        spin_sleep::sleep_until(next_frame_moment);
    }
}
