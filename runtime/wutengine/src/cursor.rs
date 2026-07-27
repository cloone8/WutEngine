//! Cursor API

/// Sets the lock/unlock state of the mouse cursor
pub fn set_lock(lock_cursor: bool) {
    crate::runtime::run_on_main_thread(move || {
        crate::window::manager::update_cursor_lock_state(lock_cursor);
    });
}

/// Locks the mouse cursor
#[inline]
pub fn lock() {
    set_lock(true);
}

/// Unlocks the mouse cursor
#[inline]
pub fn unlock() {
    set_lock(false);
}
