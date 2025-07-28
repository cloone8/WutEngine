use wutengine_util::GlobalManager;

pub(crate) static DISPLAY_MANAGER: GlobalManager<DisplayManager> = GlobalManager::new();

#[derive(Debug)]
pub(crate) struct DisplayManager {
    displays: Vec<Display>,
    main: usize,
}

impl DisplayManager {
    pub(crate) fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        let displays: Vec<Display> = event_loop
            .available_monitors()
            .map(|handle| Display::from_handle(&handle))
            .collect();

        let main_display = event_loop
            .primary_monitor()
            .and_then(|primary| {
                displays
                    .iter()
                    .position(|display| display.raw_handle == primary)
            })
            .unwrap_or(0);

        Self {
            displays,
            main: main_display,
        }
    }

    pub(crate) fn main_display(&self) -> Display {
        assert!(!self.displays.is_empty());

        self.displays[self.main].clone()
    }

    pub(crate) fn get_display(&self, id: &DisplayIdentifier) -> Option<Display> {
        self.displays
            .iter()
            .find(|display| &display.id == id)
            .cloned()
    }
}

pub fn main_display() -> Display {
    DISPLAY_MANAGER.main_display()
}

#[derive(Debug, Clone)]
pub struct Display {
    id: DisplayIdentifier,
    raw_handle: winit::monitor::MonitorHandle,
    video_modes: Vec<winit::monitor::VideoModeHandle>,
}

impl Display {
    pub fn id(&self) -> &DisplayIdentifier {
        &self.id
    }

    fn from_handle(handle: &winit::monitor::MonitorHandle) -> Self {
        Self {
            id: DisplayIdentifier(handle.clone()),
            raw_handle: handle.clone(),
            video_modes: handle.video_modes().collect(),
        }
    }

    pub(crate) fn get_mode_handle(
        &self,
        mode: VideoMode,
    ) -> Option<winit::monitor::VideoModeHandle> {
        self.video_modes
            .iter()
            .find(|mode_handle| VideoMode::from(*mode_handle) == mode)
            .cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayIdentifier(pub(crate) winit::monitor::MonitorHandle);

impl core::fmt::Display for DisplayIdentifier {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0
            .name()
            .unwrap_or_else(|| "<unknown>".to_string())
            .fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VideoMode {
    pub size: (u32, u32),
    pub bits: u16,
    pub refresh_rate_mhz: u32,
}

impl core::fmt::Display for VideoMode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let hz_float: f64 = (self.refresh_rate_mhz as f64) / 1000.0;

        write!(
            f,
            "{}x{} @ {} Hz ({} bpc)",
            self.size.0, self.size.1, hz_float, self.bits
        )
    }
}

impl<'a> From<&'a winit::monitor::VideoModeHandle> for VideoMode {
    fn from(value: &'a winit::monitor::VideoModeHandle) -> Self {
        Self {
            size: value.size().into(),
            bits: value.bit_depth(),
            refresh_rate_mhz: value.refresh_rate_millihertz(),
        }
    }
}
