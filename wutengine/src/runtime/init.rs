use alloc::sync::Arc;
use core::sync::atomic::Ordering;
use std::collections::HashMap;
use std::path::PathBuf;

use winit::error::EventLoopError;
use wutengine_util::InitOnce;

use crate::entity;
use crate::graphics;
use crate::runtime::EVENT_LOOP_PROXY;
use crate::runtime::MainThreadEvent;
use crate::runtime::Runtime;
use crate::runtime::WUTENGINE_RUNNING;
use crate::system;
use crate::window;

use super::SystemManifest;

/// An error while starting the WutEngine runtime with [run]
#[derive(Debug, derive_more::Error, derive_more::Display, derive_more::From)]
pub enum RuntimeStartErr {
    /// Another WutEngine runtime was already started in this process
    #[display("Another WutEngine runtime was already started in this process")]
    AlreadyRunning,

    /// Error running the winit event loop
    #[display("Error running the winit event loop: {_0}")]
    EventLoop(EventLoopError),
}

/// How often the frame loop runs
#[derive(Debug, Clone, Copy, Default)]
pub enum FrameFrequency {
    /// Render frames as fast as possible. Default for games
    #[default]
    Fast,

    /// Render frames at least once per `X` seconds, or when the OS requests an update
    WaitAtMost(f32),

    /// Render frames only when the OS requests an update
    WaitIndefinitely,
}

/// The configuration used to start the WutEngine runtime
#[derive(Debug, Clone)]
pub struct InitRuntimeConfig {
    /// The path to a config file, used for population the initial values of the [crate::config] module
    pub config_file: Option<PathBuf>,

    /// Hard-coded config overrides. Applied after reading the config file from [Self::config_file], and
    /// thus overrides its values
    pub config_overrides: HashMap<String, crate::config::toml::Value>,

    /// How the engine handles frame updates
    pub frame_frequency: FrameFrequency,
}

impl Default for InitRuntimeConfig {
    fn default() -> Self {
        Self {
            config_file: Some(PathBuf::from("wutengine.toml")),
            config_overrides: Default::default(),
            frame_frequency: Default::default(),
        }
    }
}

/// Data only relevant before/during application initialization in [winit::application::ApplicationHandler::resumed]
pub(super) struct InitializationData {
    pub(super) post_start_callback: Option<Box<dyn FnOnce()>>,
}

/// Starts and runs the WutEngine runtime. MUST be called from the main thread
///
/// Can only be called once per process
pub fn run(
    config: InitRuntimeConfig,
    post_start: Option<Box<dyn FnOnce()>>,
) -> Result<(), Box<RuntimeStartErr>> {
    if WUTENGINE_RUNNING.swap(true, Ordering::AcqRel) {
        return Err(Box::new(RuntimeStartErr::AlreadyRunning));
    }

    wutengine_util::set_cur_thread_as_main_thread();

    crate::event::init(|| {
        //TODO
    });

    // Initialize the config manager early, so all other managers and engine systems
    // can read from it to configure themselves
    let mut logs = crate::config::init_and_load(config.config_file.as_deref());

    for (key, val) in config.config_overrides {
        if let Err(e) = crate::config::set_raw(&key, val) {
            logs.push((
                log::Level::Warn,
                format!("Failed to set config override `{key}` due to error: {e}"),
            ));
        }
    }

    // Then the logger, so we can receive proper feedback
    initialize_logger(logs);

    log::info!("Starting WutEngine");

    #[cfg(feature = "development_overlay")]
    {
        use crate::development_overlay::ConfigOverlay;

        crate::development_overlay::init(Some(|_| {
            crate::runtime::request_frame();
        }));

        crate::development_overlay::add_development_overlay_window(ConfigOverlay::default());
        crate::development_overlay::add_development_overlay_window(
            crate::profiling::development_overlay::ProfilingOverlay::default(),
        );
    }

    let main_thread_pool = wutengine_thread::init_thread_pool();

    let mut runtime = Runtime {
        frame_pacer: window::pacer::FramePacer::default(),
        initialization_data: Some(Box::new(InitializationData {
            post_start_callback: post_start,
        })),
        entity_manager: entity::initialize(),
        systems: system::SystemManager::new(),
        draw_commands: graphics::initialize_command_queue(),
        overlay_passes: Vec::new(),
        frame_frequency: config.frame_frequency,
        async_pool: main_thread_pool,
        on_exit_requested_handlers: Vec::new(),
        on_exit_handlers: Vec::new(),
    };

    runtime.systems.build_schedule(SystemManifest::empty());

    log::debug!("Final schedule:\n{}", runtime.systems.dump());

    window::manager::init();
    crate::input::init();
    crate::physics2d::init();
    crate::physics3d::init();
    crate::audio::init();
    crate::world::init();

    let event_loop = winit::event_loop::EventLoop::<MainThreadEvent>::with_user_event()
        .build()
        .map_err(|e| Box::new(e.into()))?;

    let event_loop_proxy = event_loop.create_proxy();

    InitOnce::init(&EVENT_LOOP_PROXY, event_loop_proxy);

    let control_flow = match runtime.frame_frequency {
        FrameFrequency::Fast => winit::event_loop::ControlFlow::Poll,
        FrameFrequency::WaitAtMost(_) => winit::event_loop::ControlFlow::Wait,
        FrameFrequency::WaitIndefinitely => winit::event_loop::ControlFlow::Wait,
    };

    event_loop.set_control_flow(control_flow);

    event_loop
        .run_app(&mut runtime)
        .map_err(|e| Box::new(e.into()))?;

    Ok(())
}

fn initialize_logger(queued_messages: Vec<(log::Level, String)>) {
    let default_internal_level = crate::config::try_get("wutengine.log.default_internal_level")
        .unwrap_or(if cfg!(debug_assertions) {
            log::LevelFilter::Info
        } else {
            log::LevelFilter::Warn
        });

    let default_external_level = crate::config::try_get("wutengine.log.default_external_level")
        .unwrap_or(if cfg!(debug_assertions) {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        });

    let logger = wutengine_logger::ModuleLogger::new(
        wutengine_logger::LoggerFactory {
            create_logger: Box::new(log_factory_fn),
        },
        default_internal_level,
        default_external_level,
    );

    let result = log::set_boxed_logger(Box::new(logger));

    if result.is_ok() {
        // `set_boxed_logger` returns an error if the global logger was already
        // initialized. Therefore, if we _dont_ get an error, we can assume that
        // we have to do all the initialization.

        // We set the max level to the static max level, because the individual
        // subsystem loggers can have their own individual filters
        log::set_max_level(log::STATIC_MAX_LEVEL);
    } else {
        // Logger was already initialized, so we can safely log here
        log::info!("A logger was already initialized, so not setting WutEngine logger");
    }

    for (level, msg) in queued_messages {
        log::log!(level, "Queued log: {msg}");
    }
}

//TODO: Can remove option from target?
fn log_factory_fn(filter: log::LevelFilter, target: &str, is_internal: bool) -> Arc<dyn log::Log> {
    _ = target;

    Arc::from(simplelog::TermLogger::new(
        filter,
        simplelog::ConfigBuilder::new()
            .set_location_level(if is_internal {
                log::LevelFilter::Trace
            } else {
                log::LevelFilter::Debug
            })
            .set_target_level(log::LevelFilter::Error)
            .build(),
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Auto,
    ) as Box<dyn log::Log>)
}
