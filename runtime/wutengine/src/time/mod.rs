//! Clocks: game time, real time, fixed time and the frame counter

use core::sync::atomic::AtomicBool;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering;
use core::time::Duration;

use wutengine_sync::thread_context;

#[derive(Debug)]
struct TimeState {
    /// Scaled game time in nanoseconds
    game_time: AtomicU64,

    /// Scaled game delta time in nanoseconds
    game_delta: AtomicU64,

    /// Unscaled real time in nanoseconds
    real_time: AtomicU64,

    /// Unscaled real delta time in nanoseconds
    real_delta: AtomicU64,

    /// Time in fixed-step increments in nanoseconds
    fixed_time: AtomicU64,

    /// Fixed-step delta in nanoseconds
    fixed_delta: AtomicU64,

    /// Current fixed step number
    fixed_step_num: AtomicU64,

    /// Current frame number
    frame_num: AtomicU64,

    /// Flag that marks that the engine is currently running inside
    /// a fixed step
    in_fixed_step: AtomicBool,
}

impl TimeState {
    const fn new() -> Self {
        Self {
            game_time: AtomicU64::new(0),
            game_delta: AtomicU64::new(0),
            real_time: AtomicU64::new(0),
            real_delta: AtomicU64::new(0),
            fixed_time: AtomicU64::new(0),
            fixed_delta: AtomicU64::new(0),
            fixed_step_num: AtomicU64::new(0),
            frame_num: AtomicU64::new(0),
            in_fixed_step: AtomicBool::new(false),
        }
    }
}

/// The current time state
static TIME_STATE: TimeState = TimeState::new();

/// Returns the game time in seconds. Scaled by the time scale, and stopped while the game is paused
#[must_use]
#[inline]
pub fn time() -> f64 {
    Duration::from_nanos(time_nanos()).as_secs_f64()
}

/// Returns the exact game time in nanoseconds. See [`time`]
#[must_use]
#[inline]
pub fn time_nanos() -> u64 {
    thread_context::dbg_assert_frame_context();

    if in_fixed_step_region() {
        fixed_time_nanos()
    } else {
        TIME_STATE.game_time.load(Ordering::Relaxed)
    }
}

/// Returns the game time this frame took in seconds. Scaled by the time scale, and zero while the game is paused.
/// Inside the fixed update, returns the fixed step instead
#[must_use]
#[inline]
pub fn delta() -> f32 {
    Duration::from_nanos(delta_nanos()).as_secs_f32()
}

/// Returns the exact game time this frame took in nanoseconds. See [`delta`]
#[must_use]
#[inline]
pub fn delta_nanos() -> u64 {
    thread_context::dbg_assert_frame_context();

    if in_fixed_step_region() {
        fixed_delta_nanos()
    } else {
        TIME_STATE.game_delta.load(Ordering::Relaxed)
    }
}

/// Returns the real time in seconds. Not scaled and never paused
#[must_use]
#[inline]
pub fn unscaled_time() -> f64 {
    Duration::from_nanos(unscaled_time_nanos()).as_secs_f64()
}

/// Returns the exact real time in nanoseconds. See [`unscaled_time`]
#[must_use]
#[inline]
pub fn unscaled_time_nanos() -> u64 {
    thread_context::dbg_assert_frame_context();

    TIME_STATE.real_time.load(Ordering::Relaxed)
}

/// Returns the real time this frame took in seconds. Not scaled and never paused
#[must_use]
#[inline]
pub fn unscaled_delta() -> f32 {
    Duration::from_nanos(unscaled_delta_nanos()).as_secs_f32()
}

/// Returns the exact real time this frame took in nanoseconds. See [`unscaled_delta`]
#[must_use]
#[inline]
pub fn unscaled_delta_nanos() -> u64 {
    thread_context::dbg_assert_frame_context();

    TIME_STATE.real_delta.load(Ordering::Relaxed)
}

/// Returns the fixed time in seconds: the exact sum of the fixed steps actually run
#[must_use]
#[inline]
pub fn fixed_time() -> f64 {
    Duration::from_nanos(fixed_time_nanos()).as_secs_f64()
}

/// Returns the exact fixed time in nanoseconds. See [`fixed_time`]
#[must_use]
#[inline]
pub fn fixed_time_nanos() -> u64 {
    thread_context::dbg_assert_frame_context();

    TIME_STATE.fixed_time.load(Ordering::Relaxed)
}

/// Returns the length of one fixed step in seconds
#[must_use]
#[inline]
pub fn fixed_delta() -> f32 {
    Duration::from_nanos(fixed_delta_nanos()).as_secs_f32()
}

/// Returns the exact length of one fixed step in nanoseconds. See [`fixed_delta`]
#[must_use]
#[inline]
pub fn fixed_delta_nanos() -> u64 {
    thread_context::dbg_assert_frame_context();

    TIME_STATE.fixed_delta.load(Ordering::Relaxed)
}

/// Returns the number of fixed steps run so far
#[must_use]
#[inline]
pub fn fixed_step_num() -> u64 {
    thread_context::dbg_assert_frame_context();

    TIME_STATE.fixed_step_num.load(Ordering::Relaxed)
}

/// Returns the number of the current frame. Monotonic, and also counts non-rendered frames
#[must_use]
#[inline]
pub fn frame_num() -> u64 {
    thread_context::dbg_assert_frame_context();

    TIME_STATE.frame_num.load(Ordering::Relaxed)
}

/// Sets the fixed delta. If set to `0`, stops running fixed steps altogether
pub(crate) fn set_fixed_delta(fixed_delta_nanos: u64) {
    thread_context::assert_main_context();

    assert_ne!(0, fixed_delta_nanos, "Cannot set a 0 fixed delta");

    let in_fixed_step = TIME_STATE.in_fixed_step.load(Ordering::Relaxed);

    assert!(
        !in_fixed_step,
        "Cannot update fixed delta in the fixed step"
    );

    TIME_STATE
        .fixed_delta
        .store(fixed_delta_nanos, Ordering::Relaxed);
}

/// Starts the fixed steps region, which changes what some of the time functions return.
/// Must not already be in a fixed steps region
pub(crate) fn start_fixed_steps() {
    thread_context::assert_main_context();

    let already_in_fixed_step = TIME_STATE.in_fixed_step.swap(true, Ordering::Relaxed);

    assert!(!already_in_fixed_step, "Engine already running fixed steps");

    assert_ne!(0, fixed_delta_nanos(), "Fixed delta cannot be 0");
}

#[inline]
fn in_fixed_step_region() -> bool {
    TIME_STATE.in_fixed_step.load(Ordering::Relaxed)
}

/// Ends the fixed steps region
/// Must be in a fixed steps region for this call to be valid
pub(crate) fn end_fixed_steps() {
    thread_context::assert_main_context();

    let was_in_fixed_step = TIME_STATE.in_fixed_step.swap(false, Ordering::Relaxed);

    assert!(was_in_fixed_step, "Engine was not running fixed steps");
}

/// Begins a single fixed step in the fixed steps region
pub(crate) fn begin_fixed_step() {
    thread_context::assert_main_context();

    assert!(
        in_fixed_step_region(),
        "Engine not inside fixed steps region"
    );

    TIME_STATE.fixed_time.fetch_add(
        TIME_STATE.fixed_delta.load(Ordering::Relaxed),
        Ordering::Relaxed,
    );

    TIME_STATE.fixed_step_num.fetch_add(1, Ordering::Relaxed);
}

/// Begins a new frame
pub(crate) fn begin_frame(real_timestamp_nanos: u64, game_delta_nanos: u64) {
    profiling::function_scope!();

    thread_context::assert_main_context();

    assert!(!in_fixed_step_region(), "Engine still in fixed step region");

    // Update real time values
    let prev_real_timestamp = TIME_STATE
        .real_time
        .swap(real_timestamp_nanos, Ordering::Relaxed);

    debug_assert!(
        real_timestamp_nanos >= prev_real_timestamp,
        "New real timestamp should be equal to or larger than current"
    );

    let real_delta = real_timestamp_nanos - prev_real_timestamp;
    TIME_STATE.real_delta.store(real_delta, Ordering::Relaxed);

    // Update game time values
    TIME_STATE
        .game_time
        .fetch_add(game_delta_nanos, Ordering::Relaxed);
    TIME_STATE
        .game_delta
        .store(game_delta_nanos, Ordering::Relaxed);

    // Increment frame number
    TIME_STATE.frame_num.fetch_add(1, Ordering::Relaxed);
}
