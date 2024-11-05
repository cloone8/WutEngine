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
    /// Start time of the frame
    pub frame_start: Instant,

    /// Delta time as measured from the previous frame
    pub delta: f32,

    /// The configured fixed delta time for this frame, as
    /// used by the physics updates
    pub fixed_delta: f32,
}

impl Time {
    /// Gets the current time struct
    #[inline(always)]
    pub fn get() -> Self {
        unsafe { GLOBAL_TIME.0.get().read().assume_init() }
    }

    /// Sets the new timings
    #[inline(always)]
    unsafe fn set(new: Time) {
        unsafe {
            GLOBAL_TIME.0.get().write(MaybeUninit::new(new));
        }
    }

    /// Initializes the time struct to a valid value.
    /// Must be called once before the runtime has started, and only once.
    pub(crate) unsafe fn initialize(fixed_timestep: f32) {
        unsafe {
            Self::set(Self {
                frame_start: Instant::now(),
                delta: 0.0,
                fixed_delta: fixed_timestep,
            });
        }
    }

    /// Updates the global time struct to the current time, and the given fixed
    /// timestep.
    /// Also sets the delta to the difference between the previous and current frame start
    /// times
    ///
    /// # Safety
    ///
    /// The caller must ensure that no other thread
    /// is currently trying to update or read the time in any way.
    /// The best way to ensure this, is to only call this at a point in the
    /// frame lieftime where no systems are running.
    pub(crate) unsafe fn update_to_now(fixed_timestep: f32) {
        log::trace!("Updating global time");
        let cur_time = Instant::now();
        let prev_time = Self::get();

        unsafe {
            Self::set(Time {
                frame_start: cur_time,
                delta: cur_time.duration_since(prev_time.frame_start).as_secs_f32(),
                fixed_delta: fixed_timestep,
            });
        }
    }
}
