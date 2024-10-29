use std::any::Any;
use std::time::Instant;

use crate::component::{Component, Context};
use crate::log;
use crate::time::Time;

const NUM_SAMPLES: usize = 1000;

/// Framerate counter component. Requires [framerate_counter_system] to be present.
///
/// Every second, logs the average FPS and frametime of the last second with level `info`
#[derive(Debug)]
pub struct FramerateCounter {
    prev_report_time: Instant,
    frametimes: [f32; NUM_SAMPLES],
    frame_index: usize,
}

impl FramerateCounter {
    /// Creates a new [FramerateCounter]
    pub fn new() -> Self {
        Self {
            prev_report_time: Instant::now(),
            frametimes: [f32::NAN; NUM_SAMPLES],
            frame_index: 0,
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
        self.record_frametime();

        let cur_time = Time::get().frame_start;

        let duration_since_last_report =
            cur_time.duration_since(self.prev_report_time).as_secs_f64();

        if duration_since_last_report >= 1.0 {
            self.make_report();
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

impl FramerateCounter {
    fn record_frametime(&mut self) {
        self.frametimes[self.frame_index] = Time::get().delta;
        self.frame_index = (self.frame_index + 1) % NUM_SAMPLES;
    }

    fn make_report(&mut self) {
        let average = average(&self.frametimes) * 1000.0;

        let mut sorted = Vec::from_iter(self.frametimes.iter().copied().filter(|n| !n.is_nan()));
        sorted.sort_by(f32::total_cmp);
        sorted.reverse();

        // log::info!("{:?}", sorted);

        let low_10 = low_x(&sorted, 10.0) * 1000.0;
        let low_1 = low_x(&sorted, 1.0) * 1000.0;
        let low_01 = low_x(&sorted, 0.1) * 1000.0;

        log::info!(
            "Frametime: avg.: {:.2}ms ({:.1} FPS) - 10%: {:.2}ms - 1%: {:.2}ms - 0.1%: {:.2}ms",
            average,
            1000.0 / average,
            low_10,
            low_1,
            low_01
        );
    }
}

fn average(nums: &[f32]) -> f32 {
    let mut count = 0;
    let mut sum = 0.0;

    for &n in nums {
        if !n.is_nan() {
            count += 1;
            sum += n;
        }
    }

    if count == 0 {
        0.0
    } else {
        sum / count as f32
    }
}

fn low_x(nums: &[f32], x: f32) -> f32 {
    assert!(x <= 100.0);

    let to_count = (NUM_SAMPLES as f32 * (x / 100.0)) as usize;

    assert!(to_count > 0);

    let mut count = 0;
    let mut sum = 0.0;

    for n in nums.iter().copied().filter(|n| !n.is_nan()).take(to_count) {
        count += 1;
        sum += n;
    }

    sum / (count as f32)
}
