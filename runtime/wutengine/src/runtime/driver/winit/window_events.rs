use winit::event::WindowEvent;
use wutengine_renderer::surfaces;

use crate::log;
use crate::runtime::driver::winit::AppPhase;
use crate::runtime::driver::winit::WindowData;
use crate::runtime::driver::winit::WutEngineApp;
use crate::runtime::frame::FrameType;
use crate::time;
use crate::window;
use crate::window::Window;

impl WutEngineApp {
    pub(super) fn window_scale_factor_changed(&self, window: Window, event: &WindowEvent) {
        let WindowEvent::ScaleFactorChanged { scale_factor, .. } = event else {
            unreachable!();
        };

        profiling::scope!("ScaleFactorChanged");

        log::debug!("Window {window} scale factor changed to {scale_factor}");

        let window_data = self.native_windows.get(&window).expect("Missing window");

        window::update_window_snapshot(
            window,
            super::create_snapshot_from_native(window_data.native.as_ref()),
        );
    }
    pub(super) fn window_surface_resized(&self, window: Window, event: &WindowEvent) {
        let WindowEvent::SurfaceResized(physical_size) = event else {
            unreachable!();
        };

        profiling::scope!("SurfaceResized");

        log::debug!(
            "Window {window} resized to {}x{}",
            physical_size.width,
            physical_size.height
        );

        let window_data = self.native_windows.get(&window).expect("Missing window");

        window::update_window_snapshot(
            window,
            super::create_snapshot_from_native(window_data.native.as_ref()),
        );

        surfaces::change_surface_size(window_data.surface, (*physical_size).into());
    }

    pub(super) fn window_close_requested(window: Window) {
        profiling::scope!("CloseRequested");

        log::debug!("Close requested for window {window}");

        // Forward to our own closing function
        window.close();
    }

    pub(super) fn window_destroyed(window: Window) {
        profiling::scope!("Destroyed");

        log::debug!("Native window for window {window} destroyed");

        // We'll just "request" the close here so the window goes through our normal
        // cleanup path later
        window.close();
    }

    pub(super) fn window_redraw_requested(&mut self, window: Window) {
        profiling::scope!("RedrawRequested");

        let should_run_frame = match self.take_frame_stamp(window) {
            Some(redraw_request_stamp) => {
                // For frame `N`, we've sent redraw requests to all `M` windows.
                // If we've already drawn frame `N`, that means `time::frame_num()` now
                // returns at least `N + 1` because we've advanced the frame count while
                // rendering that frame.
                redraw_request_stamp == time::frame_num()
            }
            None => {
                // Winit asked for a redraw outside of our normal flow. This could be due to various OS related
                // things, so for safety we just run our frame loop.
                true
            }
        };

        if should_run_frame && let AppPhase::Running { frame_state } = &mut self.phase {
            frame_state.run_frame(FrameType::Continuous);
            self.request_redraws(time::frame_num());
        }
    }

    /// Request a redraw for all windows that don't already have an outstanding redraw request
    pub(super) fn request_redraws(&mut self, frame: u64) {
        for data in self.native_windows.values_mut() {
            if data.frame_stamp.is_none() {
                // No outstanding redraw request yet, send one
                data.frame_stamp = Some(frame);
                data.native.request_redraw();
            }
        }
    }

    /// Takes the outstanding redraw stamp for the given window and resets it to [`None`].
    /// If this returns [`None`], either the window is unknown or [`winit`] sent the redraw request
    /// unasked
    pub(super) fn take_frame_stamp(&mut self, window: Window) -> Option<u64> {
        match self.native_windows.get_mut(&window) {
            Some(WindowData { frame_stamp, .. }) => frame_stamp.take(),
            _ => None,
        }
    }
}
