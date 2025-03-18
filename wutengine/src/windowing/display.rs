//! Information about the displays available to WutEngine

use core::cmp::Ordering;
use std::sync::OnceLock;

use itertools::Itertools;
use winit::event_loop::ActiveEventLoop;
use winit::monitor::{MonitorHandle, VideoModeHandle};

static DISPLAYS: OnceLock<AvailableDisplays> = OnceLock::new();

/// The collection of available displays
#[derive(Debug, Clone)]
pub struct AvailableDisplays {
    displays: Vec<Display>,
    primary: Option<usize>,
}

impl AvailableDisplays {
    /// Returns the primary display. If the primary display could not be properly
    /// determined, returns the first display.
    pub fn primary(&self) -> &Display {
        self.primary
            .map(|idx| &self.displays[idx])
            .unwrap_or(&self.displays[0])
    }

    /// Returns all displays
    pub fn all(&self) -> &[Display] {
        &self.displays
    }
}

/// An available display
#[derive(Debug, Clone)]
pub struct Display {
    pub(crate) handle: MonitorHandle,
    pub(super) modes: Vec<VideoModeHandle>,
    pub(super) largest_mode: VideoModeHandle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct DisplayId(pub(crate) MonitorHandle);

impl Display {
    pub fn id(&self) -> DisplayId {
        DisplayId(self.handle.clone())
    }

    /// Returns the name of the display
    pub fn name(&self) -> String {
        self.handle.name().unwrap()
    }

    /// Returns the supported exclusive fullscreen video modes
    pub fn modes(&self) -> &[VideoModeHandle] {
        &self.modes
    }

    /// Returns largest video mode supported by this display,
    /// as determined by resolution primarily, and then refresh rate
    pub fn largest_mode(&self) -> VideoModeHandle {
        self.largest_mode.clone()
    }
}

/// Obtains information about the available monitors
/// from the event loop, and stores that in the global
/// storage
pub(crate) fn configure(event_loop: &ActiveEventLoop) {
    log::trace!("Identifying monitors");

    let available = event_loop.available_monitors().collect_vec();
    let primary = event_loop.primary_monitor();

    let mut displays = AvailableDisplays {
        displays: available.into_iter().map(map_display).collect(),
        primary: None,
    };

    if let Some(primary) = primary {
        for (i, display) in displays.displays.iter().enumerate() {
            if display.handle == primary {
                displays.primary = Some(i);
                break;
            }
        }
    }

    DISPLAYS
        .set(displays)
        .expect("Displays already configured!");
}

fn map_display(handle: MonitorHandle) -> Display {
    let modes = handle.video_modes().collect_vec();

    let mut largest_mode: Option<&VideoModeHandle> = None;

    for mode in &modes {
        if let Some(cur_largest) = largest_mode {
            if better_mode(mode, cur_largest) {
                largest_mode = Some(mode);
            }
        } else {
            largest_mode = Some(mode);
        }
    }

    Display {
        handle,
        largest_mode: largest_mode
            .expect("Could not determine best video mode!")
            .clone(),
        modes,
    }
}

/// Returns whether `mode` is a "better" (larger, faster) video mode when compared to
/// `compared_to`
fn better_mode(mode: &VideoModeHandle, compared_to: &VideoModeHandle) -> bool {
    let resolution_cmp = mode.size().cmp(&compared_to.size());
    let refresh_cmp = mode
        .refresh_rate_millihertz()
        .cmp(&compared_to.refresh_rate_millihertz());

    match resolution_cmp {
        Ordering::Less => false,
        Ordering::Equal => match refresh_cmp {
            Ordering::Less => false,
            Ordering::Equal => false,
            Ordering::Greater => true,
        },
        Ordering::Greater => true,
    }
}

/// Returns the available displays
pub fn available_displays() -> AvailableDisplays {
    DISPLAYS
        .get()
        .expect("Displays not yet initialized!")
        .clone()
}

pub fn get_display(id: &DisplayId) -> Option<Display> {
    available_displays()
        .displays
        .iter()
        .find(|disp| disp.id() == *id)
        .cloned()
}
