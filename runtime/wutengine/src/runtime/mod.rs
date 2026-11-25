//! Engine runtime lifecycle

#[cfg(target_arch = "wasm32")]
use core::cell::RefCell;
use core::panic::Location;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use std::panic;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::event_loop::EventLoop;
use wutengine_log::FileLogger;
use wutengine_sync::thread_context;

use crate::log;
use crate::panic_handling;
use crate::runtime::winit_app::WutEngineApp;

mod frame;
mod startup;
mod winit_app;

/// Startup options used to start the [WutEngine runtime](crate::runtime)
/// Not public API, should be used through the [`wutengine_macro::main`] macro
#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct StartupOptions {
    /// Organization name, used for storing/loading data
    pub org_name: &'static str,

    /// Application name, used for storing/loading data
    pub app_name: &'static str,
}

cfg_select! {
    target_arch = "wasm32" => {
        type StartRet = ();
    }
    _ => {
        type StartRet = !;
    }
}

#[cfg(target_arch = "wasm32")]
struct WebInitArgs {
    options: StartupOptions,
    app_entrypoint: Box<dyn FnOnce() + 'static>,
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WEB_STARTUP_ARGS: RefCell<Option<WebInitArgs>> = const { RefCell::new(None) };
}

/// Stores the startup options in a global static, so it can be used by
#[cfg(target_arch = "wasm32")]
#[doc(hidden)]
pub fn _wutengine_web_init(options: StartupOptions, app_entrypoint: impl FnOnce() + 'static) {
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

/// Starts the WutEngine runtime. Should be called by the JavaScript-side loader, after `init()` was
/// called.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start_web(canvas: Option<web_sys::HtmlCanvasElement>) {
    let Some(startup_args) = WEB_STARTUP_ARGS.take() else {
        panic!("Web init was not called");
    };

    #[expect(
        clippy::used_underscore_items,
        reason = "Hidden for external users only"
    )]
    {
        _wutengine_entry(&startup_args.options, startup_args.app_entrypoint, canvas);
    }
}

/// Starts the WutEngine runtime. Must be called once, from the main thread
/// Not public API, should be used through the [`wutengine_macro::main`] macro
#[doc(hidden)]
pub fn _wutengine_entry(
    options: &StartupOptions,
    app_entrypoint: impl FnOnce() + 'static,
    #[cfg(target_arch = "wasm32")] canvas: Option<web_sys::HtmlCanvasElement>,
) -> StartRet {
    static INIT: AtomicBool = AtomicBool::new(false);

    if INIT.swap(true, Ordering::Relaxed) {
        panic!("Runtime already initialized");
    }

    thread_context::mark_main_thread();

    panic::set_hook(Box::new(panic_handling::panic_hook));

    let basic_logger = cfg_select! {
        target_arch = "wasm32" => {{
            // Web logs to `console` API
            Box::new(wutengine_log::BrowserConsoleLogger::new())
        }}
        _ => {{
            // Native logs to terminal
            Box::new(wutengine_log::TerminalLogger::new(
                wutengine_log::OutputMode::Stdout,
                wutengine_log::ColorMode::Auto,
            ))
        }}
    };

    log::init(
        log::LogSettings::with_default_categories(),
        vec![basic_logger],
    );

    if !cfg!(target_arch = "wasm32") {
        // Web can't log to files. Native can
        match FileLogger::new(options.org_name, options.app_name, "engine") {
            Ok(fl) => log::add_output(Box::new(fl)),
            Err(e) => {
                log::error!("Failed to create file logger: {e}");
            }
        }
    }

    // Run the app entrypoint first, so that users can have chance to run registrations
    app_entrypoint();

    let event_loop = match EventLoop::builder().build() {
        Ok(el) => el,
        Err(e) => {
            panic!("Failed to build an event loop: {e}"); //TODO: Replace with fatal error once we have an API for it
        }
    };

    #[allow(unused_mut, reason = "Used for web")]
    let mut app = WutEngineApp::new();

    #[cfg(target_arch = "wasm32")]
    {
        app.set_main_canvas(canvas);
    }

    if let Err(e) = event_loop.run_app(app) {
        panic!("Error while running app event loop: {e}"); //TODO: Replace with fatal error later
    }

    #[cfg(not(target_arch = "wasm32"))]
    exit_now(0);
}

/// Requests that the WutEngine runtime quits as soon as possible.
/// This call might still return
pub fn quit() {
    //TODO: This will probably be handled more neatly by the runtime later, with hooks and stuff
    exit_now(0);
}

/// Immediatly quits the WutEngine runtime with an error message and exit code `100`
#[track_caller]
pub fn fatal_error(message: &str) -> ! {
    let location = Location::caller();
    log::error!("FATAL ERROR @ {location}: {message}");

    exit_now(100); // 100 chosen arbitrarily
}

/// Exit the application with the given exit code
pub(crate) fn exit_now(code: i32) -> ! {
    log::info!("Quitting runtime with exit code {code}");
    log::flush();

    cfg_select! {
        target_arch = "wasm32" => {{
            // On the web we throw a custom exception, because `process::exit` does not work there
            use js_sys::Reflect;
            use wasm_bindgen::JsValue;

            let err = js_sys::Error::new(format!("WutEngine quit with exit code {code}").as_str());

            err.set_name("WutEngineQuit");
            Reflect::set(&err, &JsValue::from_str("exitCode"), &JsValue::from(code))
                .expect("Failed to set exitcode");

            wasm_bindgen::throw_val(err.into());
        }}
        _ => {{
            use std::process;

            process::exit(code);
        }}
    }
}
