use core::num::Saturating;
use std::any::Any;
use std::time::Instant;

use crate::component::{Component, Context};
use crate::log;
use crate::time::Time;

/// Framerate counter component. Requires [framerate_counter_system] to be present.
///
/// Every second, logs the average FPS and frametime of the last second with level `info`
#[derive(Debug)]
pub struct FramerateCounter {
    prev_report_time: Instant,
    num_frames: Saturating<usize>,
}

impl FramerateCounter {
    /// Creates a new [FramerateCounter]
    pub fn new() -> Self {
        Self {
            prev_report_time: Instant::now(),
            num_frames: Saturating(0),
        }
    }
}

impl Default for FramerateCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for FramerateCounter {
    fn update(&mut self, _context: &mut Context) {
        self.num_frames += 1;

        let cur_time = Time::get().frame_start;

        let duration_since_last_report =
            cur_time.duration_since(self.prev_report_time).as_secs_f64();

        if duration_since_last_report >= 1.0 {
            let fps = self.num_frames.0 as f64 / duration_since_last_report;

            log::info!("FPS: {}, avg. frametime: {}ms", fps, (1.0 / fps) * 1000.0);

            self.num_frames = Saturating(0);
            self.prev_report_time = cur_time;
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
