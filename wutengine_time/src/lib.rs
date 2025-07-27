//! Time management and functions

use core::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use wutengine_util::GlobalManager;

use crate::atomic::AtomicF32;

mod atomic;

/// The global time manager
static TIME_MANAGER: GlobalManager<TimeManager> = GlobalManager::new();

/// Internal [TimeManager] fields
struct TimeManagerInternal {
    /// Previous frame start-time in real time
    prev_frame: Instant,

    /// The latest requested time scale
    requested_time_scale: f32,

    /// The latest requested fixed delta
    requested_fixed_delta: f32,

    /// The current fixed-timestep accumulator
    fixed_accumulator: f32,

    /// The target delta time
    target_delta_time: f32,
}

/// The main time manager. Takes care of updating the delta times (both scaled/unscaled) when requested by the engine runtime.
struct TimeManager {
    /// Internal values, meant to be accessed synchronized only
    internal: Mutex<TimeManagerInternal>,

    /// The time scale used
    time_scale: AtomicF32,

    /// The amount of frames that have passed in total since application start
    frame_num: AtomicUsize,

    /// The current time since application start
    time: AtomicF32,

    /// Delta time as measured from the previous frame
    delta: AtomicF32,

    /// The current time since application start, unscaled by the configured time scale
    unscaled_time: AtomicF32,

    /// Delta time as measured from the previous frame
    unscaled_delta: AtomicF32,

    /// The current time since application start
    fixed_time: AtomicF32,

    /// The configured fixed delta time for this frame, as
    /// used by the physics updates
    fixed_delta: AtomicF32,
}

impl TimeManager {
    /// Creates a new time manager with the specified config
    fn new(target_delta: f32, fixed_delta: f32) -> Self {
        Self {
            internal: Mutex::new(TimeManagerInternal {
                prev_frame: Instant::now(),
                requested_time_scale: 1.0,
                target_delta_time: target_delta,
                requested_fixed_delta: fixed_delta,
                fixed_accumulator: 0.0,
            }),
            time_scale: AtomicF32::new(1.0),
            frame_num: AtomicUsize::new(0),

            time: AtomicF32::new(0.0),
            unscaled_time: AtomicF32::new(0.0),
            delta: AtomicF32::new(target_delta),
            unscaled_delta: AtomicF32::new(target_delta),

            fixed_time: AtomicF32::new(0.0),
            fixed_delta: AtomicF32::new(fixed_delta),
        }
    }
}

/// Initializes the time manager with the specified config
pub fn init(target_delta: f32, fixed_delta: f32) {
    GlobalManager::init(&TIME_MANAGER, TimeManager::new(target_delta, fixed_delta));
}

/// Returns the current time scaling factor
#[inline(always)]
pub fn time_scale() -> f32 {
    TIME_MANAGER.time_scale.load(Ordering::Acquire)
}

/// Returns the current frame number
#[inline(always)]
pub fn frame_num() -> usize {
    TIME_MANAGER.frame_num.load(Ordering::Acquire)
}

/// Returns the time at the beginning of this frame since the start of the engine runtime (in seconds)
#[inline(always)]
pub fn time() -> f32 {
    TIME_MANAGER.time.load(Ordering::Acquire)
}

/// Returns the unscaled time at the beginning of this frame since the start of the engine runtime (in seconds)
///
/// Use for any code that needs to be unaffected by pauses, slow-motion, etc.
#[inline(always)]
pub fn unscaled_time() -> f32 {
    TIME_MANAGER.unscaled_time.load(Ordering::Acquire)
}

/// The delta time for this frame
#[inline(always)]
pub fn delta() -> f32 {
    TIME_MANAGER.delta.load(Ordering::Acquire)
}

/// The unscaled delta time for this frame
///
/// Use for any code that needs to be unaffected by pauses, slow-motion, etc.
#[inline(always)]
pub fn unscaled_delta() -> f32 {
    TIME_MANAGER.unscaled_delta.load(Ordering::Acquire)
}

/// The time at the beginning of the current fixed timestep since the start of the engine runtime (in seconds)
#[inline(always)]
pub fn fixed_time() -> f32 {
    TIME_MANAGER.fixed_time.load(Ordering::Acquire)
}

/// The current fixed delta time
#[inline(always)]
pub fn fixed_delta() -> f32 {
    TIME_MANAGER.fixed_delta.load(Ordering::Acquire)
}

/// Updates the fixed timestep to the new value, starting next frame
pub fn set_fixed_delta(fixed_delta: f32) {
    if fixed_delta < 0.0 || !fixed_delta.is_normal() {
        log::error!("Cannot set fixed delta to invalid time span {fixed_delta}");
        return;
    }

    TIME_MANAGER.internal.lock().unwrap().requested_fixed_delta = fixed_delta;
}

/// Updates the time scale to the new value, starting next frame
pub fn set_time_scale(time_scale: f32) {
    if time_scale < 0.0 || !time_scale.is_finite() {
        log::error!("Cannot set time scale to invalid factor {time_scale}");
        return;
    }

    TIME_MANAGER.internal.lock().unwrap().requested_time_scale = time_scale;
}

/// Updates the target timestep to the new value, starting next frame
pub fn set_target_delta(target_delta: f32) {
    if target_delta < 0.0 || !target_delta.is_normal() {
        log::error!("Cannot set target delta to invalid time span {target_delta}");
        return;
    }

    TIME_MANAGER.internal.lock().unwrap().target_delta_time = target_delta;
}

/// Updates time configuration to the last requested values.
/// Returns a tuple of `(new_time_scale, new_fixed_delta)`
fn update_time_config(tm: &TimeManagerInternal) -> (f32, f32) {
    let new_time_scale = tm.requested_time_scale;
    let new_fixed_delta = tm.requested_fixed_delta;

    TIME_MANAGER
        .time_scale
        .store(new_time_scale, Ordering::Release);

    TIME_MANAGER
        .fixed_delta
        .store(new_fixed_delta, Ordering::Release);

    (new_time_scale, new_fixed_delta)
}

/// Called by the engine runtime to update the current time according to the
/// the specified `now`. Returns the amount of fixed updates to run this frame
#[profiling::function]
pub fn update_frame(now: Instant) -> usize {
    let mut time_manager_internal = TIME_MANAGER.internal.lock().unwrap();

    // Set new scale / delta based on the last requested values
    let (new_time_scale, new_fixed_delta) = update_time_config(&time_manager_internal);

    // Update the frame counter
    TIME_MANAGER.frame_num.fetch_add(1, Ordering::AcqRel);

    // Real-time
    let unclamped_time_delta = now
        .duration_since(time_manager_internal.prev_frame)
        .as_secs_f32();

    // Sometimes no updates are sent if the window was inactive for a long time, or if there
    // was a major frame-time hitch. In these cases we clamp the delta
    // to the target delta, to make sure the simulation doesn't try to jump an insane
    // amount in one frame
    let time_since_prev_frame = if unclamped_time_delta > 1.0 {
        log::warn!(
            "Clamping frame delta time to target frame time ({}) because it was longer than one second ({}).",
            time_manager_internal.target_delta_time,
            unclamped_time_delta
        );

        time_manager_internal.target_delta_time
    } else {
        unclamped_time_delta
    };

    // Calculate the deltas and the number of fixed steps this frame
    let unscaled_delta = time_since_prev_frame;
    TIME_MANAGER
        .unscaled_time
        .fetch_add(unscaled_delta, Ordering::Release);
    TIME_MANAGER
        .unscaled_delta
        .store(unscaled_delta, Ordering::Release);

    let scaled_delta = unscaled_delta * new_time_scale;
    let new_time = TIME_MANAGER.time.fetch_add(scaled_delta, Ordering::Release);
    TIME_MANAGER.delta.store(scaled_delta, Ordering::Release);

    let mut fixed_accumulator = time_manager_internal.fixed_accumulator + scaled_delta;

    let fixed_steps_to_run = (fixed_accumulator / new_fixed_delta).floor() as usize;
    fixed_accumulator -= fixed_steps_to_run as f32 * new_fixed_delta;

    time_manager_internal.fixed_accumulator = fixed_accumulator;

    // As the new fixed time, we clamp set it to the current "normal" time to prevent
    // them going out of sync due to an accumulation of floating point errors
    TIME_MANAGER.fixed_time.store(new_time, Ordering::Release);

    time_manager_internal.prev_frame = now;

    fixed_steps_to_run
}

/// Called by the engine runtime after fixed updates in between frames to update the fixed timestamp
#[profiling::function]
pub fn update_fixed() {
    let unscaled_delta = TIME_MANAGER.fixed_delta.load(Ordering::Acquire);
    let scaled_delta = unscaled_delta * TIME_MANAGER.time_scale.load(Ordering::Acquire);

    TIME_MANAGER
        .fixed_time
        .fetch_add(scaled_delta, Ordering::AcqRel);
}
