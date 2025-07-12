//! Time management and related functionality for WutEngine

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use std::time::Instant;

/// The current global time values, shared across
/// all systems and threads.
///
/// # Safety
///
/// Storing this unsafely should be fine, because it
/// is should only ever be updated by the runtime
/// right before starting a new frame. At this point, as we do not give
/// out references to this static, no other systems should have
/// access to the global time memory location.
static GLOBAL_TIME: SyncUnsafeCell<MaybeUninit<Time>> =
    SyncUnsafeCell(UnsafeCell::new(MaybeUninit::zeroed()));

#[derive(Debug)]
#[repr(transparent)]
struct SyncUnsafeCell<T>(UnsafeCell<T>);

unsafe impl<T: Sync> Sync for SyncUnsafeCell<T> {}

/// Current time values
#[derive(Debug, Clone, Copy)]
pub struct Time {
    /// Start time of the entire level
    pub level_start: Instant,

    /// Start time of the frame
    pub frame_start: Instant,

    /// The amount of frames that have passed in this level
    pub frame_num: usize,

    /// Delta time as measured from the previous frame
    pub delta: f32,

    /// The configured fixed delta time for this frame, as
    /// used by the physics updates
    pub fixed_delta: f32,

    /// The fixed timestep accumulator
    pub fixed_accumulator: f32,
}

#[inline(always)]
pub fn delta() -> f32 {
    get().delta
}

#[inline(always)]
pub fn level_start() -> Instant {
    get().level_start
}

#[inline(always)]
pub fn frame_start() -> Instant {
    get().frame_start
}

#[inline(always)]
pub fn frame_num() -> usize {
    get().frame_num
}

#[inline(always)]
pub fn fixed_delta() -> f32 {
    get().fixed_delta
}

/// Gets the current time struct
#[inline(always)]
fn get() -> Time {
    unsafe { GLOBAL_TIME.0.get().read().assume_init() }
}

/// Sets the new timings
#[inline(always)]
unsafe fn set(new: Time) {
    unsafe {
        GLOBAL_TIME.0.get().write(MaybeUninit::new(new));
    }
}

#[inline(always)]
unsafe fn _set_scene_start(new: Instant) {
    let mut prev_time = get();

    prev_time.level_start = new;

    unsafe {
        set(prev_time);
    }
}

/// Initializes the time struct to a valid value.
/// Must be called once before the runtime has started, and only once.
pub(crate) unsafe fn init(fixed_timestep: f32) {
    unsafe {
        set(Time {
            level_start: Instant::now(),
            frame_num: 0,
            frame_start: Instant::now(),
            delta: 0.0,
            fixed_delta: fixed_timestep,
            fixed_accumulator: 0.0,
        });
    }
}

/// Updates the global time struct to the current time, and returns the amount
/// of fixed timesteps to run
///
/// # Safety
///
/// The caller must ensure that no other thread
/// is currently trying to update or read the time in any way.
/// The best way to ensure this, is to only call this at a point in the
/// frame lieftime where no systems are running.
pub(crate) unsafe fn update_to_now() -> usize {
    log::trace!("Updating global time");

    let cur_time = Instant::now();
    let prev_time = get();
    let delta = cur_time.duration_since(prev_time.frame_start).as_secs_f32();
    let mut fixed_accumulator = prev_time.fixed_accumulator + delta;

    let fixed_steps_to_run = fixed_accumulator.floor();
    fixed_accumulator -= fixed_steps_to_run;

    unsafe {
        set(Time {
            level_start: prev_time.level_start,
            frame_num: prev_time.frame_num.wrapping_add(1),
            frame_start: cur_time,
            delta,
            fixed_delta: prev_time.fixed_delta,
            fixed_accumulator,
        });
    }

    fixed_steps_to_run as usize
}
