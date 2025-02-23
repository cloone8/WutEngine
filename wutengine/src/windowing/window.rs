//! The wrapper for a WutEngine [Window], and related functionality
use core::ops::{Deref, DerefMut};

use crate::util::markers::NonSendSync;

/// A WutEngine Window
#[derive(Debug)]
pub struct Window {
    /// The native window handle
    pub(crate) os_window: WinitWindow,

    /// The cached window data
    pub(crate) window_data: WindowData,
}

impl Window {
    /// Creates a new [Window] wrapper struct from an existing native [winit::window::Window]
    pub(crate) fn new(winit: winit::window::Window) -> Self {
        Self {
            window_data: WindowData::from_winit(&winit),
            os_window: WinitWindow::new(winit),
        }
    }

    /// Updates the [Window] wrapper data by pulling the data from the native window
    pub(crate) fn update(&mut self) {
        self.window_data.update(&self.os_window);
    }
}

/// The native Winit window. Can be used by plugins
#[derive(Debug)]
pub struct WinitWindow {
    /// The raw window
    pub(crate) raw: winit::window::Window,
    _marker: NonSendSync,
}

impl WinitWindow {
    /// Creates a new Winit window wrapper from an actual [winit::window::Window]
    pub(crate) fn new(winit: winit::window::Window) -> Self {
        Self {
            raw: winit,
            _marker: NonSendSync::new(),
        }
    }
}

impl Deref for WinitWindow {
    type Target = winit::window::Window;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl DerefMut for WinitWindow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw
    }
}

/// Cached data for a [Window]
#[derive(Debug, Clone)]
pub struct WindowData {
    /// The window size in pixels (x, y)
    pub size: (u32, u32),
}

impl WindowData {
    /// Creates a new [WindowData] struct by pulling all relevant info from the given
    /// [winit::window::Window]
    pub(crate) fn from_winit(_winit: &winit::window::Window) -> Self {
        Self {
            size: _winit.inner_size().into(),
        }
    }

    /// Updates an existing [WindowData] struct by pulling all relevant info from the given
    /// [winit::window::Window]
    pub(crate) fn update(&mut self, _winit: &winit::window::Window) {
        self.size = _winit.inner_size().into();
    }
}
