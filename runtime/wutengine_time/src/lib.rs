#![doc = include_str!("../README.md")]

use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use wutengine_util::InitOnce;

/// The amount of nanoseconds in one second
pub const NANOS_PER_SECOND: u64 = 1_000_000_000;

/// The global time manager
static TIME_MANAGER: InitOnce<TimeManager, false> = InitOnce::new_checked();

/// Internal [`TimeManager`] fields
/// All times are in nanoseconds, unless stated otherwise
struct TimeManagerInternal {
    /// Previous frame start-time in real time
    prev_frame: Instant,

    /// The latest requested time scale
    requested_time_scale: f64,

    /// The maximum frame time in nanoseconds before
    /// the time manager caps it
    maximum_frame_time: u64,

    /// The latest requested fixed delta
    requested_fixed_delta: u64,

    /// The current fixed-timestep accumulator
    fixed_accumulator: u64,

    /// The target delta time
    target_delta_time: u64,
}

/// The main time manager. Takes care of updating the delta times (both scaled/unscaled) when requested by the engine runtime.
/// All times are in nanoseconds, unless stated otherwise
struct TimeManager {
    /// Internal values, meant to be accessed synchronized only
    internal: Mutex<TimeManagerInternal>,

    /// The time scale used, stored as a bitcast [f64] using [`f64::to_bits`] and [`f64::from_bits`]
    time_scale: AtomicU64,

    /// The amount of frames that have passed in total since application start
    frame_num: AtomicUsize,

    /// The current time since application start
    time: AtomicU64,

    /// Delta time as measured from the previous frame
    delta: AtomicU64,

    /// The current time since application start, unscaled by the configured time scale
    unscaled_time: AtomicU64,

    /// Delta time as measured from the previous frame
    unscaled_delta: AtomicU64,

    /// The current time since application start
    fixed_time: AtomicU64,

    /// The configured fixed delta time for this frame, as
    /// used by the physics updates
    fixed_delta: AtomicU64,
}

impl TimeManager {
    /// Creates a new time manager with the specified config
    fn new(target_delta_nanos: u64, fixed_delta_nanos: u64, frame_time_cap: u64) -> Self {
        Self {
            internal: Mutex::new(TimeManagerInternal {
                prev_frame: Instant::now(),
                requested_time_scale: 1.0,
                target_delta_time: target_delta_nanos,
                maximum_frame_time: frame_time_cap,
                requested_fixed_delta: fixed_delta_nanos,
                fixed_accumulator: 0,
            }),
            time_scale: AtomicU64::new(1.0_f64.to_bits()),
            frame_num: AtomicUsize::new(0),

            time: AtomicU64::new(0),
            unscaled_time: AtomicU64::new(0),
            delta: AtomicU64::new(target_delta_nanos),
            unscaled_delta: AtomicU64::new(target_delta_nanos),

            fixed_time: AtomicU64::new(0),
            fixed_delta: AtomicU64::new(fixed_delta_nanos),
        }
    }
}

/// Time manager configuration
#[derive(Debug)]
struct TimeManagerConfig {
    /// The target framerate in nanoseconds
    target_frame_time: u64,

    /// The fixed timestep in nanoseconds
    fixed_timestep: u64,

    /// The maximum frametime before the engine starts to treat a frame as an "unwanted pause"
    maximum_frame_time: u64,
}

impl TimeManagerConfig {
    /// Validates the config and replaces any invalid values with valid ones
    fn validate(&mut self) {
        let mut validated = Self::default();

        if self.target_frame_time != 0 {
            validated.target_frame_time = self.target_frame_time;
        } else {
            log::warn!(
                "Invalid target frame time of {} given. Setting to default",
                self.target_frame_time
            );
        }

        if self.fixed_timestep != 0 {
            validated.fixed_timestep = self.fixed_timestep;
        } else {
            log::warn!(
                "Invalid fixed timestep of {} given. Setting to default",
                self.fixed_timestep
            );
        }

        if self.maximum_frame_time != 0 && self.maximum_frame_time > validated.target_frame_time {
            validated.maximum_frame_time = self.maximum_frame_time;
        } else {
            log::warn!(
                "Invalid maximum frametime of {} given. Should be higher than 0 and higher than the target frame time of {}. Setting to default",
                self.maximum_frame_time,
                validated.target_frame_time
            );
            validated.maximum_frame_time = u64::max(
                validated.maximum_frame_time,
                validated.target_frame_time + 1,
            );
        }

        *self = validated;
    }
}

impl Default for TimeManagerConfig {
    fn default() -> Self {
        Self {
            target_frame_time: NANOS_PER_SECOND / 60,
            fixed_timestep: NANOS_PER_SECOND / 50,
            maximum_frame_time: NANOS_PER_SECOND,
        }
    }
}

/// Initializes the time manager with the specified config
#[doc(hidden)]
pub fn init() {
    let mut config = TimeManagerConfig::default();
    config.validate();

    InitOnce::init(
        &TIME_MANAGER,
        TimeManager::new(
            config.target_frame_time,
            config.fixed_timestep,
            config.maximum_frame_time,
        ),
    );
}

/// Returns the current time scaling factor
#[inline]
pub fn time_scale() -> f64 {
    f64::from_bits(TIME_MANAGER.time_scale.load(Ordering::Acquire))
}

/// Returns the current frame number
#[inline]
pub fn frame_num() -> usize {
    TIME_MANAGER.frame_num.load(Ordering::Acquire)
}

/// Returns the time at the beginning of this frame, since engine startup
///
/// Time is returned in nanoseconds
#[inline]
pub fn time_nanos() -> u64 {
    TIME_MANAGER.time.load(Ordering::Acquire)
}

/// Returns the time at the beginning of this frame, since engine startup
///
/// Time is returned in seconds, as a double
#[inline]
#[expect(clippy::cast_precision_loss, reason = "Inherent to operation")]
pub fn time64() -> f64 {
    (time_nanos() as f64) / (NANOS_PER_SECOND as f64)
}

/// Returns the time at the beginning of this frame, since engine startup
///
/// Time is returned in seconds, as a float
#[inline]
#[expect(clippy::cast_precision_loss, reason = "Inherent to operation")]
pub fn time() -> f32 {
    (time_nanos() as f32) / (NANOS_PER_SECOND as f32)
}

/// Returns the unscaled time at the beginning of this frame since the start of the engine runtime
///
/// Use for any code that needs to be unaffected by pauses, slow-motion, etc.
///
/// Time is returned in nanoseconds
#[inline]
pub fn unscaled_time_nanos() -> u64 {
    TIME_MANAGER.unscaled_time.load(Ordering::Acquire)
}

/// Returns the unscaled time at the beginning of this frame since the start of the engine runtime
///
/// Use for any code that needs to be unaffected by pauses, slow-motion, etc.
///
/// Time is returned in seconds, as a double
#[inline]
#[expect(clippy::cast_precision_loss, reason = "Inherent to operation")]
pub fn unscaled_time64() -> f64 {
    (unscaled_time_nanos() as f64) / (NANOS_PER_SECOND as f64)
}

/// Returns the unscaled time at the beginning of this frame since the start of the engine runtime
///
/// Use for any code that needs to be unaffected by pauses, slow-motion, etc.
///
/// Time is returned in seconds, as a float
#[inline]
#[expect(clippy::cast_precision_loss, reason = "Inherent to operation")]
pub fn unscaled_time() -> f32 {
    (unscaled_time_nanos() as f32) / (NANOS_PER_SECOND as f32)
}

/// The delta time for this frame
///
/// Time is returned in nanoseconds
#[inline]
pub fn delta_nanos() -> u64 {
    TIME_MANAGER.delta.load(Ordering::Acquire)
}

/// The delta time for this frame
///
/// Time is returned in seconds, as a double
#[inline]
#[expect(clippy::cast_precision_loss, reason = "Inherent to operation")]
pub fn delta64() -> f64 {
    (delta_nanos() as f64) / (NANOS_PER_SECOND as f64)
}

/// The delta time for this frame
///
/// Time is returned in seconds, as a float
#[inline]
#[expect(clippy::cast_precision_loss, reason = "Inherent to operation")]
pub fn delta() -> f32 {
    (delta_nanos() as f32) / (NANOS_PER_SECOND as f32)
}

/// The unscaled delta time for this frame
///
/// Use for any code that needs to be unaffected by pauses, slow-motion, etc.
///
/// Time is returned in nanoseconds
#[inline]
pub fn unscaled_delta_nanos() -> u64 {
    TIME_MANAGER.unscaled_delta.load(Ordering::Acquire)
}

/// The unscaled delta time for this frame
///
/// Use for any code that needs to be unaffected by pauses, slow-motion, etc.
///
/// Time is returned in seconds, as a double
#[inline]
#[expect(clippy::cast_precision_loss, reason = "Inherent to operation")]
pub fn unscaled_delta64() -> f64 {
    (unscaled_delta_nanos() as f64) / (NANOS_PER_SECOND as f64)
}

/// The unscaled delta time for this frame
///
/// Use for any code that needs to be unaffected by pauses, slow-motion, etc.
///
/// Time is returned in seconds, as a float
#[inline]
#[expect(clippy::cast_precision_loss, reason = "Inherent to operation")]
pub fn unscaled_delta() -> f32 {
    (unscaled_delta_nanos() as f32) / (NANOS_PER_SECOND as f32)
}

/// The time at the beginning of the current fixed timestep since the start of the engine runtime
///
/// Time is returned in nanoseconds
#[inline]
pub fn fixed_time_nanos() -> u64 {
    TIME_MANAGER.fixed_time.load(Ordering::Acquire)
}

/// The time at the beginning of the current fixed timestep since the start of the engine runtime
///
/// Time is returned in seconds, as a double
#[inline]
#[expect(clippy::cast_precision_loss, reason = "Inherent to operation")]
pub fn fixed_time64() -> f64 {
    (fixed_time_nanos() as f64) / (NANOS_PER_SECOND as f64)
}

/// The time at the beginning of the current fixed timestep since the start of the engine runtime
///
/// Time is returned in seconds, as a float
#[inline]
#[expect(clippy::cast_precision_loss, reason = "Inherent to operation")]
pub fn fixed_time() -> f32 {
    (fixed_time_nanos() as f32) / (NANOS_PER_SECOND as f32)
}

/// The fixed delta time for the current fixed timestep
///
/// Time is returned in nanoseconds
#[inline]
pub fn fixed_delta_nanos() -> u64 {
    TIME_MANAGER.fixed_delta.load(Ordering::Acquire)
}

/// The fixed delta time for the current fixed timestep
///
/// Time is returned in seconds, as a double
#[inline]
#[expect(clippy::cast_precision_loss, reason = "Inherent to operation")]
pub fn fixed_delta64() -> f64 {
    (fixed_delta_nanos() as f64) / (NANOS_PER_SECOND as f64)
}

/// The fixed delta time for the current fixed timestep
///
/// Time is returned in seconds, as a float
#[inline]
#[expect(clippy::cast_precision_loss, reason = "Inherent to operation")]
pub fn fixed_delta() -> f32 {
    (fixed_delta_nanos() as f32) / (NANOS_PER_SECOND as f32)
}

/// Updates the fixed timestep to the new value, starting next frame
///
/// Value is in nanoseconds (see [`NANOS_PER_SECOND`])
pub fn set_fixed_delta(fixed_delta_nanos: u64) {
    if fixed_delta_nanos == 0 {
        log::error!("Cannot set fixed delta to invalid time span {fixed_delta_nanos}");
        return;
    }

    TIME_MANAGER.internal.lock().unwrap().requested_fixed_delta = fixed_delta_nanos;
}

/// Updates the time scale to the new value, starting next frame
///
/// Value is in seconds
pub fn set_time_scale(time_scale: f64) {
    if time_scale < 0.0 || !time_scale.is_finite() {
        log::error!("Cannot set time scale to invalid factor {time_scale}");
        return;
    }

    TIME_MANAGER.internal.lock().unwrap().requested_time_scale = time_scale;
}

/// Updates the target timestep to the new value, starting next frame
///
/// Value is in nanoseconds (see [`NANOS_PER_SECOND`])
pub fn set_target_delta(target_delta_nanos: u64) {
    if target_delta_nanos == 0 {
        log::error!("Cannot set target delta to invalid time span {target_delta_nanos}");
        return;
    }

    TIME_MANAGER.internal.lock().unwrap().target_delta_time = target_delta_nanos;
}

/// Updates the maximum frame time before clamping to the new value, starting next frame
///
/// Value is in nanoseconds (see [`NANOS_PER_SECOND`])
pub fn set_max_frame_time(max_frame_time_nanos: u64) {
    if max_frame_time_nanos == 0 {
        log::error!("Cannot set max frame time to invalid time span {max_frame_time_nanos}");
        return;
    }

    TIME_MANAGER.internal.lock().unwrap().maximum_frame_time = max_frame_time_nanos;
}

/// Updates time configuration to the last requested values.
///
/// Returns a tuple of `(new_time_scale_factor, new_fixed_delta_nanos)`
fn update_time_config(tm: &TimeManagerInternal) -> (f64, u64) {
    let new_time_scale = tm.requested_time_scale;
    let new_fixed_delta = tm.requested_fixed_delta;

    TIME_MANAGER
        .time_scale
        .store(new_time_scale.to_bits(), Ordering::Release);

    TIME_MANAGER
        .fixed_delta
        .store(new_fixed_delta, Ordering::Release);

    (new_time_scale, new_fixed_delta)
}

/// Called by the engine runtime to update the current time according to the
/// the specified `now`. Returns the amount of fixed updates to run this frame
pub fn update_frame(now: Instant) -> u64 {
    profiling::function_scope!();

    let mut time_manager_internal = TIME_MANAGER.internal.lock().unwrap();

    // Set new scale / delta based on the last requested values
    let (new_time_scale, new_fixed_delta) = update_time_config(&time_manager_internal);

    assert!(
        new_time_scale.is_sign_positive(),
        "Time scale must be positive"
    );

    // Update the frame counter
    TIME_MANAGER.frame_num.fetch_add(1, Ordering::AcqRel);

    // Real-time
    let unclamped_time_delta_nanos = now
        .duration_since(time_manager_internal.prev_frame)
        .as_nanos();

    // Sometimes no updates are sent if the window was inactive for a long time, or if there
    // was a major frame-time hitch. In these cases we clamp the delta
    // to the target delta, to make sure the simulation doesn't try to jump an insane
    // amount in one frame
    let time_since_prev_frame = if unclamped_time_delta_nanos
        > u128::from(time_manager_internal.maximum_frame_time)
    {
        log::warn!(
            "Clamping frame delta time ({}) to target frame time ({}) because it was longer than the maximum frame time ({}).",
            unclamped_time_delta_nanos,
            time_manager_internal.target_delta_time,
            time_manager_internal.maximum_frame_time
        );

        time_manager_internal.target_delta_time
    } else {
        // Safe cast because we know the unclamped delta is less than NANOS_PER_SECONDS, which fits in a u64
        u64::try_from(unclamped_time_delta_nanos).expect("Should fit")
    };

    // Calculate the deltas and the number of fixed steps this frame
    let unscaled_delta = time_since_prev_frame;
    TIME_MANAGER
        .unscaled_time
        .fetch_add(unscaled_delta, Ordering::Release);
    TIME_MANAGER
        .unscaled_delta
        .store(unscaled_delta, Ordering::Release);

    #[expect(
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        reason = "Checked"
    )]
    let scaled_delta = ((unscaled_delta as f64) * new_time_scale) as u64;
    let new_time = TIME_MANAGER.time.fetch_add(scaled_delta, Ordering::Release);
    TIME_MANAGER.delta.store(scaled_delta, Ordering::Release);

    let mut fixed_accumulator = time_manager_internal.fixed_accumulator + scaled_delta;

    // Integer division automatically floors this
    let fixed_steps_to_run = fixed_accumulator / new_fixed_delta;
    fixed_accumulator %= new_fixed_delta; // Accumulator is now the remainder of the non-stepped time

    time_manager_internal.fixed_accumulator = fixed_accumulator;

    // As the new fixed time, we clamp set it to the current "normal" time to prevent
    // them going out of sync due to an accumulation of floating point errors
    TIME_MANAGER.fixed_time.store(new_time, Ordering::Release);

    time_manager_internal.prev_frame = now;

    fixed_steps_to_run
}

/// Called by the engine runtime after fixed updates in between frames to update the fixed timestamp
pub fn update_fixed() {
    profiling::function_scope!();

    let unscaled_delta = TIME_MANAGER.fixed_delta.load(Ordering::Acquire);
    let time_scale = f64::from_bits(TIME_MANAGER.time_scale.load(Ordering::Acquire));

    assert!(time_scale.is_sign_positive(), "Time scale must be positive");

    #[expect(
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        reason = "Checked"
    )]
    let scaled_delta = ((unscaled_delta as f64) * time_scale) as u64;

    TIME_MANAGER
        .fixed_time
        .fetch_add(scaled_delta, Ordering::AcqRel);
}
