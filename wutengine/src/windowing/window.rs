use core::ops::{Deref, DerefMut};

use crate::util::markers::NonSendSync;

#[derive(Debug)]
pub struct Window {
    pub(crate) os_window: WinitWindow,
    pub(crate) window_data: WindowData,
}

impl Window {
    pub(crate) fn new(winit: winit::window::Window) -> Self {
        Self {
            window_data: WindowData::from_winit(&winit),
            os_window: WinitWindow::new(winit),
        }
    }

    pub(crate) fn update(&mut self) {
        self.window_data.update(&self.os_window);
    }
}

#[derive(Debug)]
pub struct WinitWindow {
    pub(crate) raw: winit::window::Window,
    marker: NonSendSync,
}

impl WinitWindow {
    pub(crate) fn new(winit: winit::window::Window) -> Self {
        Self {
            raw: winit,
            marker: NonSendSync::new(),
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

#[derive(Debug, Clone)]
pub struct WindowData {
    pub size: (u32, u32),
}

impl WindowData {
    pub(crate) fn from_winit(_winit: &winit::window::Window) -> Self {
        Self {
            size: _winit.inner_size().into(),
        }
    }

    pub(crate) fn update(&mut self, _winit: &winit::window::Window) {
        self.size = _winit.inner_size().into();
    }
}
