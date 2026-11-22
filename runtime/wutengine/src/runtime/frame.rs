//! The main frame/step functions

use core::hint;
use core::time::Duration;

use crate::log;
use crate::time;

cfg_select! {
    target_arch = "wasm32" => {
        use web_time::Instant;
    }
    _ => {
        use std::time::Instant;
    }
}

#[derive(Debug)]
pub(super) struct FrameState {
    clock_start: Instant,
    time_scale: f64,
    paused: bool,
    nominal_frame_time_nanos: u64,
}

/// What was happening before the call to [`FrameState::run_frame`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FrameType {
    /// A previous frame. We're running continuously
    Continuous,

    /// An idle period where we were not simulating frames, for any reason.
    AfterIdle,
}

impl FrameState {
    pub(super) fn new(nominal_frame_time: Duration) -> Self {
        Self {
            clock_start: Instant::now(),
            nominal_frame_time_nanos: u64::try_from(nominal_frame_time.as_nanos())
                .expect("Nominal frame time too large"),
            time_scale: 1.0,
            paused: false,
        }
    }
}

impl FrameState {
    #[expect(
        clippy::needless_pass_by_ref_mut,
        reason = "Will be used later, and making non-mut will make to too easy to use poorly until then"
    )]
    pub(super) fn run_frame(&mut self, frame_type: FrameType) {
        profiling::function_scope!();
        log::trace!("Running frame");

        let cur_clock = Instant::now();

        let (real_time, game_delta) = self.compute_times(cur_clock, frame_type);

        // Updates all non-fixed timers so they become visible to callers
        time::begin_frame(real_time, game_delta);

        Self::fixed_update();

        //TODO: The rest of the owl
    }

    fn fixed_update() {
        profiling::function_scope!();

        let game_time = time::time_nanos();
        let fixed_time = time::fixed_time_nanos();

        time::start_fixed_steps();

        debug_assert!(
            game_time >= fixed_time,
            "Fixed time should not have advanced further than game time"
        );

        let mut accumulator = game_time - fixed_time;

        let fixed_delta = time::fixed_delta_nanos();

        while accumulator >= fixed_delta {
            profiling::scope!("fixed step");

            time::begin_fixed_step();

            log::trace!("Running one fixed update step");

            accumulator -= fixed_delta;
        }

        time::end_fixed_steps();
    }

    /// Returns `(real_time, game_delta)`
    fn compute_times(&self, clock_now: Instant, frame_type: FrameType) -> (u64, u64) {
        // Compute real times
        let real_time = u64::try_from(clock_now.duration_since(self.clock_start).as_nanos())
            .expect("Real time overflow");

        let prev_real_time = time::unscaled_time_nanos();
        let real_delta = real_time - prev_real_time;

        // Compute game times
        let game_delta = self.compute_game_delta(real_delta, time::frame_num() == 0, frame_type);

        (real_time, game_delta)
    }

    fn compute_game_delta(
        &self,
        real_delta: u64,
        is_first_frame: bool,
        frame_type: FrameType,
    ) -> u64 {
        const MAX_DELTA_NANOS: u64 = 250_000_000; // 0.25s

        if self.paused {
            return 0;
        }

        let base_delta = if frame_type == FrameType::AfterIdle || is_first_frame {
            // Only for after-idle frames and the first frame
            hint::cold_path();

            self.nominal_frame_time_nanos
        } else {
            // Default path
            u64::min(real_delta, MAX_DELTA_NANOS) //TODO: Make configurable, maybe?
        };

        ((base_delta as f64) * self.time_scale).round() as u64
    }
}
