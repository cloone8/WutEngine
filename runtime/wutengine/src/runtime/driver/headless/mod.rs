//! Headless driver. Only runs the frame loop without any renderer interaction

use core::time::Duration;
use std::thread;
use std::time::Instant;

use crate::runtime::frame::FrameState;
use crate::runtime::frame::FrameType;

#[derive(Debug)]
pub(crate) struct HeadlessApp {
    frame_state: FrameState,
    tick_interval: Duration,
}

impl HeadlessApp {
    /// Creates a new headless runner, running `tick_rate` game updates per second
    pub(crate) fn new(tick_rate: f64) -> Self {
        assert!(
            tick_rate.is_normal() && tick_rate.is_sign_positive(),
            "Invalid tickrate given"
        );

        let tick_interval = Duration::from_secs_f64(1.0 / tick_rate);

        Self {
            tick_interval,
            frame_state: FrameState::new(tick_interval),
        }
    }

    pub(crate) fn run(&mut self) {
        // The first frame runs right away. Each next deadline is the previous one plus the
        // interval, so sleep overshoot doesn't add up over frames
        let mut next_tick = Instant::now();

        #[expect(clippy::infinite_loop, reason = "Stopped by runtime functions")]
        loop {
            self.frame_state.run_frame(FrameType::Continuous);

            next_tick = next_tick
                .checked_add(self.tick_interval)
                .expect("Invalid next tick");

            let now = Instant::now();

            if let Some(remaining) = next_tick.checked_duration_since(now) {
                if !remaining.is_zero() {
                    thread::sleep(remaining);
                }
            } else if now.duration_since(next_tick) > self.tick_interval {
                // More than one interval late: re-anchor to now instead of running a burst of
                // frames to catch up. The game delta's clamp already limits catch-up fixed steps
                next_tick = now;
            }
        }
    }
}
