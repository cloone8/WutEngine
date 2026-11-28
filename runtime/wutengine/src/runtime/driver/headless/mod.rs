//! Headless driver. Only runs the frame loop without any renderer interaction

use core::assert_matches;
use core::task::Waker;
use core::time::Duration;
use std::thread;
use std::time::Instant;

use wutengine_executor::ThreadLocalExecutor;

use crate::runtime;
use crate::runtime::exit_now;
use crate::runtime::frame::FrameState;
use crate::runtime::frame::FrameType;
use crate::runtime::hooks;

mod init;

#[derive(Debug)]
pub(crate) struct HeadlessApp {
    tick_interval: Duration,
    main_thread_executor: ThreadLocalExecutor,
    phase: AppPhase,
}

#[derive(Debug)]
enum AppPhase {
    /// [`HeadlessApp::run`] has not been called yet
    NotStarted,

    /// Driver is running
    Running(FrameState),
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
            main_thread_executor: ThreadLocalExecutor::new(),
            phase: AppPhase::NotStarted,
        }
    }

    pub(crate) fn run(&mut self) -> ! {
        assert_matches!(
            self.phase,
            AppPhase::NotStarted,
            "Cannot run headless driver twice"
        );

        // Run initialization
        self.main_thread_executor.block_on(init::initialize());

        hooks::run_started_hooks();

        // Start running the frames/ticks
        self.phase = AppPhase::Running(FrameState::new(self.tick_interval));

        // The first frame runs right away. Each next deadline is the previous one plus the
        // interval, so sleep overshoot doesn't add up over frames
        let mut next_tick = Instant::now();

        profiling::finish_frame!(); // So we can track the entire startup as a single "frame"

        while !runtime::should_quit() {
            self.main_thread_executor.run_ready(Waker::noop());

            let AppPhase::Running(frame_state) = &mut self.phase else {
                unreachable!();
            };

            frame_state.run_frame(FrameType::Continuous);

            profiling::finish_frame!();

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

        hooks::run_quitting_hooks();
        exit_now(0);
    }
}
