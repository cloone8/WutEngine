use wutengine_util_macro::unique_id_type32;

use crate::graphics;

use super::manager::display_info::DisplayExclusiveFullscreenMode;

unique_id_type32! {
    /// The handle to a display
    pub Display
}

impl Display {
    /// The primary display for the system
    #[inline]
    pub fn primary() -> Self {
        crate::window::manager::primary_display().expect("No displays connected")
    }

    /// Returns the available displays
    #[inline]
    pub fn available() -> Vec<Self> {
        crate::window::manager::get_displays()
    }

    /// Returns the exclusive fullscreen modes supported by the system
    #[inline]
    pub fn exclusive_fullscreen_modes(self) -> Vec<DisplayExclusiveFullscreenMode> {
        if !graphics::active_config()
            .backend
            .supports_exclusive_fullscreen()
        {
            return Vec::new();
        }

        crate::window::manager::get_display_and(self, |disp| {
            disp.map(|disp| disp.video_modes.clone())
                .unwrap_or_default()
        })
    }

    /// Returns the OS name for the display, if available
    #[inline]
    pub fn name(self) -> Option<String> {
        crate::window::manager::get_display_and(self, |disp| {
            disp.and_then(|disp| disp.name.clone())
        })
    }

    /// Returns the resolution in pixels of the display
    #[inline]
    pub fn resolution(self) -> (u32, u32) {
        crate::window::manager::get_display_and(self, |disp| disp.map_or((0, 0), |disp| disp.size))
    }

    /// Returns the system refresh rate for this display
    #[inline]
    pub fn refresh_rate_millihertz(self) -> u32 {
        crate::window::manager::get_display_and(self, |disp| {
            disp.map_or(0, |disp| disp.refresh_rate_millihertz)
        })
    }

    /// Returns the OS scale factor for this display
    #[inline]
    pub fn scale_factor(self) -> f64 {
        crate::window::manager::get_display_and(self, |disp| {
            disp.map_or(1.0, |disp| disp.scaling_factor)
        })
    }

    /// Returns whether this display is the primary system display
    #[inline]
    pub fn is_primary(self) -> bool {
        crate::window::manager::get_display_and(self, |disp| {
            disp.is_some_and(|disp| disp.is_primary)
        })
    }
}
