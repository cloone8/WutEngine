//! Clocks: game time, real time, fixed time and the frame counter

/// Returns the game time in seconds. Scaled by the time scale, and stopped while the game is paused
#[must_use]
pub fn time() -> f64 {
    todo!()
}

/// Returns the exact game time in nanoseconds. See [`time`]
#[must_use]
pub fn time_nanos() -> u64 {
    todo!()
}

/// Returns the game time this frame took in seconds. Scaled by the time scale, and zero while the game is paused.
/// Inside the fixed update, returns the fixed step instead
#[must_use]
pub fn delta() -> f32 {
    todo!()
}

/// Returns the exact game time this frame took in nanoseconds. See [`delta`]
#[must_use]
pub fn delta_nanos() -> u64 {
    todo!()
}

/// Returns the real time in seconds. Not scaled and never paused
#[must_use]
pub fn unscaled_time() -> f64 {
    todo!()
}

/// Returns the exact real time in nanoseconds. See [`unscaled_time`]
#[must_use]
pub fn unscaled_time_nanos() -> u64 {
    todo!()
}

/// Returns the real time this frame took in seconds. Not scaled and never paused
#[must_use]
pub fn unscaled_delta() -> f32 {
    todo!()
}

/// Returns the exact real time this frame took in nanoseconds. See [`unscaled_delta`]
#[must_use]
pub fn unscaled_delta_nanos() -> u64 {
    todo!()
}

/// Returns the fixed time in seconds: the exact sum of the fixed steps actually run
#[must_use]
pub fn fixed_time() -> f64 {
    todo!()
}

/// Returns the exact fixed time in nanoseconds. See [`fixed_time`]
#[must_use]
pub fn fixed_time_nanos() -> u64 {
    todo!()
}

/// Returns the length of one fixed step in seconds
#[must_use]
pub fn fixed_delta() -> f32 {
    todo!()
}

/// Returns the exact length of one fixed step in nanoseconds. See [`fixed_delta`]
#[must_use]
pub fn fixed_delta_nanos() -> u64 {
    todo!()
}

/// Returns the number of fixed steps run so far
#[must_use]
pub fn fixed_step_num() -> u64 {
    todo!()
}

/// Returns the number of the current frame. Monotonic, and also counts non-rendered frames
#[must_use]
pub fn frame_num() -> u64 {
    todo!()
}
