//! Web (wasm32) platform implementation
//!
//! The web start-up happens in two steps, both driven by the JavaScript-side loader:
//! 1. The loader's `init()` runs the game's `main`, which only stores the start-up arguments ([`main`]).
//! 2. The loader then calls [`start_web`] with the canvas to render to, which runs the engine.

use core::cell::RefCell;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;

use js_sys::Reflect;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::HtmlCanvasElement;
use winit::platform::web::WindowAttributesWeb;
use winit::window::WindowAttributes;
use wutengine_log::LoggerOutput;

use crate::runtime;
use crate::runtime::StartupOptions;
use crate::window::Window;

/// Return type of the engine entry point. On the web it returns right away after storing the
/// start-up arguments, see [`main`]
pub(super) type StartRet = ();

/// Start-up arguments stored by [`main`], until [`start_web`] runs the engine with them
struct WebInitArgs {
    /// The game's start-up options
    options: StartupOptions,

    /// The game's entry point
    app_entrypoint: Box<dyn FnOnce() + 'static>,
}

thread_local! {
    /// Set by [`main`], taken by [`start_web`]
    static WEB_STARTUP_ARGS: RefCell<Option<WebInitArgs>> = const { RefCell::new(None) };

    /// The canvas passed in by the loader, if any. Taken when the main window is created
    static MAIN_CANVAS: RefCell<Option<HtmlCanvasElement>> = const { RefCell::new(None) };
}

/// Platform entry point, called from the game's `main`. Only stores the start-up arguments: the
/// engine itself is started by the loader calling [`start_web`]
pub(super) fn main(options: StartupOptions, app_entrypoint: impl FnOnce() + 'static) -> StartRet {
    static INIT: AtomicBool = AtomicBool::new(false);

    if INIT.swap(true, Ordering::Relaxed) {
        panic!("Web init already called");
    }

    WEB_STARTUP_ARGS.with(|startup_args| {
        *startup_args.borrow_mut() = Some(WebInitArgs {
            options,
            app_entrypoint: Box::new(app_entrypoint),
        });
    });
}

/// Starts the WutEngine runtime. Called by the JavaScript-side loader, after `init()` was
/// called. The main window renders to `canvas` if given, otherwise to a new canvas appended to the page
#[wasm_bindgen]
#[expect(
    unreachable_pub,
    reason = "Exported to JavaScript, not to Rust. `wasm_bindgen` requires `pub`"
)]
pub fn start_web(canvas: Option<HtmlCanvasElement>) {
    let Some(startup_args) = WEB_STARTUP_ARGS.take() else {
        panic!("Web init was not called");
    };

    MAIN_CANVAS.set(canvas);

    runtime::run(&startup_args.options, startup_args.app_entrypoint);
}

/// The log output that is available from the very start: the browser console
pub(super) fn base_log_output() -> Box<dyn LoggerOutput> {
    Box::new(wutengine_log::BrowserConsoleLogger::new())
}

/// Adds the log outputs that need the logger to be running already. The web has none
pub(super) fn add_log_outputs(_options: &StartupOptions) {}

/// Applies the platform-specific window attributes. The main window gets the loader's canvas, if
/// it passed one
pub(super) fn finish_window_attributes(
    window: Window,
    mut attributes: WindowAttributes,
) -> WindowAttributes {
    let canvas = if window.is_main() {
        MAIN_CANVAS.take()
    } else {
        None
    };

    if canvas.is_some() {
        // An existing canvas keeps the size the page gave it
        attributes.surface_size = None;
    }

    attributes.platform = Some(Box::new(
        WindowAttributesWeb::default()
            .with_canvas(canvas)
            .with_append(true),
    ));

    attributes
}

/// Exits the application with the given exit code. `process::exit` does not work on the web, so
/// this throws a `WutEngineQuit` JavaScript error carrying the exit code instead
pub(super) fn exit(code: i32) -> ! {
    let err = js_sys::Error::new(format!("WutEngine quit with exit code {code}").as_str());

    err.set_name("WutEngineQuit");
    Reflect::set(&err, &JsValue::from_str("exitCode"), &JsValue::from(code))
        .expect("Failed to set exitcode");

    wasm_bindgen::throw_val(err.into());
}
