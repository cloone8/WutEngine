#![doc = include_str!("../README.md")]

extern crate alloc;

use alloc::sync::Arc;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use std::sync::Mutex;
use wutengine_egui::LogicOutput;
use wutengine_egui::TextureMaterialMap;
use wutengine_egui::egui;
use wutengine_input::WindowIdentifier;
use wutengine_util_macro::unique_id_type32;

use wutengine_graphics::wgpu;

use wutengine_util::InitOnce;

#[doc(inline)]
pub use wutengine_egui;

unique_id_type32! {
    DevOverlayWindowId
}

/// Global [DevOverlayManager]
static DEV_OVERLAY: InitOnce<DevOverlayManager> = InitOnce::new_checked();

#[doc(hidden)]
pub fn init(repaint_callback: Option<impl Fn(egui::RequestRepaintInfo) + Send + Sync + 'static>) {
    InitOnce::init(&DEV_OVERLAY, DevOverlayManager::new(repaint_callback));
}

/// Manages the calculating and rendering of the development overlay
pub(crate) struct DevOverlayManager {
    /// Whether the overlay should be active
    active: AtomicBool,

    /// The last target window for the dev overlay
    last_target_window: Mutex<WindowIdentifier>,

    /// The egui window
    egui_window: Mutex<Option<Box<wutengine_egui::EguiWindow>>>,

    /// The [egui::Context]
    egui_context: egui::Context,

    /// The materials for each texture
    textures: wutengine_egui::TextureMaterialMap,

    /// All registered dev windows
    windows: Mutex<Vec<DevOverlayWindow>>,
}

/// A single development overlay window, injected through [add_development_overlay_window]
struct DevOverlayWindow {
    /// The unique ID of the window
    id: DevOverlayWindowId,

    /// Whether the window should be open now
    open: bool,

    /// The implementation of the window
    window: Box<dyn DevelopmentOverlayWindow>,
}

impl DevOverlayManager {
    /// A new empty [DevOverlayManager]
    fn new(
        repaint_callback: Option<impl Fn(egui::RequestRepaintInfo) + Send + Sync + 'static>,
    ) -> Self {
        let egui_context = wutengine_egui::egui::Context::default();

        if let Some(repaint_callback) = repaint_callback {
            egui_context.set_request_repaint_callback(repaint_callback);
        }

        Self {
            active: AtomicBool::new(false),
            last_target_window: Mutex::new(WindowIdentifier::new(0)),
            egui_window: Mutex::new(None),
            egui_context,
            textures: TextureMaterialMap::default(),
            windows: Mutex::new(Vec::new()),
        }
    }
}

/// A WutEngine development overlay. Can be added to the engine using [crate::development_overlay::add_development_overlay_window]
pub trait DevelopmentOverlayWindow: Send + Sync + 'static {
    /// The name of the overlay
    fn name(&self) -> &str;

    /// An icon to show on the overlay window. Optional. Should be an emoji or something similar
    fn icon(&self) -> Option<&str> {
        None
    }

    /// Shows the UI
    fn show(&mut self, ui: &mut egui::Ui);

    /// Called when the window was either opened or closed
    fn window_state_changed(&mut self, opened: bool) {
        _ = opened;
    }
}

/// Runs the logic to draw the dev overlay, if it is active.
///
/// Returns a [std::sync::mpsc::Receiver] that will receive exactly one message when the overlay is done
/// calculating, as that is done on a different thread. When the receiver has received its message, [render_overlay] should
/// be called before another call to [run_overlay_logic] in order to render the calculated overlay
/// onto a target
pub fn run_overlay_logic(
    input_window: wutengine_input::WindowIdentifier,
    window_info: wutengine_egui::EguiWindowInfo,
    surface_size: (u32, u32),
    scale_factor: f32,
    handle_logic_output: impl FnOnce(LogicOutput) + Send + Sync + 'static,
) {
    profiling::function_scope!();

    if !is_enabled() {
        return;
    }

    let start_barrier = Arc::new(std::sync::Barrier::new(2));

    let start_barrier_clone = start_barrier.clone();
    rayon::spawn(move || {
        profiling::scope!("Run overlay logic");

        let sfc_size = (surface_size.0, surface_size.1);
        let sfc_points = (
            sfc_size.0 as f32 / scale_factor,
            sfc_size.1 as f32 / scale_factor,
        );

        let mut egui_window_lock = DEV_OVERLAY.egui_window.lock().unwrap();

        start_barrier_clone.wait(); // We know we locked the window, so when we unlock it, the overlay logic is done

        let egui_window: &mut Option<_> = &mut egui_window_lock;

        match egui_window {
            Some(window) => {
                window.input_window_identifier = input_window;
                window.window_info = window_info;
                window.surface_size_points = sfc_points;
                window.scale_factor = scale_factor;
            }
            None => {
                let mut new_egui_window = wutengine_egui::EguiWindow::new(input_window, sfc_points);

                new_egui_window.title = "WutEngine Development Overlay".to_string();
                new_egui_window.window_info = window_info;
                new_egui_window.surface_size_points = sfc_points;
                new_egui_window.scale_factor = scale_factor;

                *egui_window = Some(new_egui_window);
            }
        }

        let egui_window = egui_window.as_ref().unwrap();

        let output = egui_window.run_logic(
            &DEV_OVERLAY.egui_context,
            &DEV_OVERLAY.textures,
            dev_overlay_ui,
        );

        *DEV_OVERLAY.last_target_window.lock().unwrap() = input_window;

        drop(egui_window_lock);

        handle_logic_output(output);
    });

    // We have to at least wait until the overlay window is locked so that we do not proceed with the overlay renderpass
    // while we haven't even started with the overlay logic yet
    start_barrier.wait();
}

/// Dev overlay UI callback
fn dev_overlay_ui(ui: &mut egui::Ui) {
    let mut windows = DEV_OVERLAY.windows.lock().unwrap();

    egui::Window::new("WutEngine Development Overlay")
        .collapsible(false)
        .order(egui::Order::Background)
        .resizable(false)
        .default_open(true)
        .show(ui, |ui| {
            if windows.is_empty() {
                ui.label("No development windows registered");
                return;
            }

            for window in windows.iter_mut() {
                if ui.button(window.window.name()).clicked() {
                    window.open = !window.open;
                    window.window.window_state_changed(window.open);
                }

                let title_with_icon = window
                    .window
                    .icon()
                    .map(|icon| format!("{} {}", icon, window.window.name()));

                let title_str = match &title_with_icon {
                    Some(with_icon) => with_icon.as_str(),
                    None => window.window.name(),
                };

                egui::Window::new(title_str)
                    .id(egui::Id::new(window.id))
                    .open(&mut window.open)
                    .show(ui, |ui| {
                        window.window.show(ui);
                    });
            }
        });
}

/// Renders the current development overlay. Should be preceded by a call to [run_overlay_logic], and the returned channel should
/// have been waited on.
pub fn render_overlay_if_window_eq(
    window: &WindowIdentifier,
    target: &wgpu::Texture,
    command_encoder: &mut wgpu::CommandEncoder,
) -> bool {
    profiling::function_scope!();

    let egui_window_lock = DEV_OVERLAY.egui_window.lock().unwrap();

    let Some(egui_window) = egui_window_lock.as_ref() else {
        return false;
    };

    if *DEV_OVERLAY.last_target_window.lock().unwrap() != *window {
        return false;
    }

    let mut to_free = Vec::new();

    egui_window.render_window(target, &DEV_OVERLAY.textures, command_encoder, &mut to_free);

    DEV_OVERLAY.textures.free_removed(to_free);

    true
}

/// Enable the development overlay
#[inline]
pub fn enable() {
    set_state(true);
}

/// Disable the development overlay
#[inline]
pub fn disable() {
    set_state(false);
}

/// Enable or disable the development overlay
pub fn set_state(active: bool) {
    log::debug!(
        "Development overlay set to: {}",
        if active { "enabled" } else { "disabled" }
    );

    DEV_OVERLAY.active.store(active, Ordering::Release);
}

/// Returns whether the development overlay is currently enabled
pub fn is_enabled() -> bool {
    DEV_OVERLAY.active.load(Ordering::Acquire)
}

/// Add a new [DevelopmentOverlayWindow] to the engine
pub fn add_development_overlay_window<T: DevelopmentOverlayWindow>(window: T) {
    DEV_OVERLAY.windows.lock().unwrap().push(DevOverlayWindow {
        id: DevOverlayWindowId::new(),
        open: false,
        window: Box::new(window),
    });
}
