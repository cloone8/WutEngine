use command::{Command, OpenWindowParams};

#[doc(inline)]
pub use wutengine_core as core;

#[doc(inline)]
pub use wutengine_ecs as ecs;
use wutengine_ecs::world::World;
use wutengine_ecs::Dynamic;

#[doc(inline)]
pub use wutengine_graphics as graphics;

#[doc(inline)]
pub use wutengine_macro as macros;

use wutengine_core::System;

mod embedded {
    use include_dir::{include_dir, Dir};

    pub static SHADERS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/shaders");
}

pub mod builtins;
pub mod command;
pub mod log;
pub mod math;
pub mod plugin;
pub mod renderer;
pub mod runtime;

//NOTE: This top-level module will _not_ be logged due to level filtering difficulties. Put any logic in a submodule.

#[derive(Debug)]
pub enum EngineCommand {
    AddSystem(System<World, Command>),
    SpawnEntity(Vec<Dynamic>),
    OpenWindow(OpenWindowParams),
}

#[derive(Debug)]
pub enum WindowingEvent {
    OpenWindow(OpenWindowParams),
}

#[derive(Debug)]
pub enum EngineEvent {
    RuntimeStart,
}
