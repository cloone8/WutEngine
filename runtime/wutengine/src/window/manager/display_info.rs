use wutengine_util::assert_main_thread;

use crate::window::Display;

#[derive(Debug)]
pub(in crate::window) struct DisplayInfo {
    pub(in crate::window) handle: winit::monitor::MonitorHandle,
    pub(in crate::window) name: Option<String>,
    pub(in crate::window) size: (u32, u32),
    pub(in crate::window) refresh_rate_millihertz: u32,
    pub(in crate::window) scaling_factor: f64,
    pub(in crate::window) video_modes: Vec<DisplayExclusiveFullscreenMode>,
    pub(in crate::window) is_primary: bool,
}

impl DisplayInfo {
    pub(super) fn from_monitor_handle(
        id: Display,
        handle: winit::monitor::MonitorHandle,
        is_primary: bool,
    ) -> Self {
        profiling::function_scope!();
        assert_main_thread!();

        Self {
            name: handle.name(),
            size: (handle.size().width, handle.size().height),
            refresh_rate_millihertz: handle
                .refresh_rate_millihertz()
                .expect("Display dissapeared during configuration"),
            scaling_factor: handle.scale_factor(),
            video_modes: handle
                .video_modes()
                .map(|videomode| DisplayExclusiveFullscreenMode(id, videomode))
                .collect(),
            is_primary,
            handle,
        }
    }
}

/// A video mode for [`exclusive fullscreen`](crate::window::FullscreenMode::Exclusive). Combines
/// a target display, a resolution, refresh rate, and an amount of bits-per-color
#[derive(Debug, Clone)]
pub struct DisplayExclusiveFullscreenMode(
    pub(in crate::window) Display,
    pub(in crate::window) winit::monitor::VideoModeHandle,
);

impl DisplayExclusiveFullscreenMode {
    /// The target display for this fullscreen mode
    #[inline]
    pub const fn display(&self) -> Display {
        self.0
    }

    /// The resolution of this mode
    #[inline]
    pub fn resolution(&self) -> (u32, u32) {
        let phys_size = self.1.size();

        (phys_size.width, phys_size.height)
    }

    /// The refresh rate in millihertz for this mode
    #[inline]
    pub fn refresh_rate_millihertz(&self) -> u32 {
        self.1.refresh_rate_millihertz()
    }

    /// The available bits per color for this mode
    #[inline]
    pub fn bits_per_color(&self) -> u16 {
        self.1.bit_depth()
    }
}
