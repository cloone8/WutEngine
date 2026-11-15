//! Engine runtime lifecycle

use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;

use wutengine_log::FileLogger;

use crate::log;

/// Startup options used to start the [WutEngine runtime](crate::runtime)
#[derive(Debug)]
pub struct StartupOptions<'a> {
    /// Organization name, used for storing/loading data
    pub org_name: &'a str,

    /// Application name, used for storing/loading data
    pub app_name: &'a str,
}

/// Starts the WutEngine runtime. Must be called once, from the main thread
pub fn start(options: &StartupOptions, post_startup: impl FnOnce() + 'static) -> ! {
    static INIT: AtomicBool = AtomicBool::new(false);

    if INIT.swap(true, Ordering::Relaxed) {
        panic!("Runtime already initialized")
    }

    let basic_logger = cfg_select! {
        target_arch = "wasm32" => {{ Box::new(wutengine_log::BrowserConsoleLogger::new()) }}
        _ => {{
            Box::new(wutengine_log::TerminalLogger::new(
                wutengine_log::OutputMode::Stdout,
                wutengine_log::ColorMode::Auto,
            ))
        }}
    };

    log::init(log::LogSettings::new(), vec![basic_logger]);

    if !cfg!(target_arch = "wasm32") {
        match FileLogger::new(options.org_name, options.app_name, "engine") {
            Ok(fl) => log::add_output(Box::new(fl)),
            Err(e) => {
                log::error!("Failed to create file logger: {e}");
            }
        }
    }

    log::info!("WutEngine start-up done");

    post_startup();

    quit(0);

    unreachable!("Quit still immediately aborts for now");
}

/// Requests that the WutEngine runtime quits as soon as possible.
/// This call might still return
pub fn quit(code: i32) {
    //TODO: This will probably be handled more neatly by the runtime later, with hooks and stuff
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
