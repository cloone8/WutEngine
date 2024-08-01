use command::{Command, OpenWindowParams};

#[doc(inline)]
pub use wutengine_core as core;

#[doc(inline)]
pub use wutengine_graphics as graphics;

#[doc(inline)]
pub use wutengine_macro as macros;

use world::World;

use wutengine_core::{DynComponent, EntityId, System};

mod embedded {
    use include_dir::{include_dir, Dir};

    pub static SHADERS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/shaders");
}

pub mod command;
pub mod components;
pub mod log;
pub mod math;
pub mod plugin;
pub mod renderer;
pub mod runtime;
pub mod storage;
pub mod world;

//NOTE: This top-level module will _not_ be logged due to level filtering difficulties. Put any logic in a submodule.

#[derive(Debug)]
pub enum SystemFunction {
    Immutable(for<'a> fn(&mut Command, &'a World<'a>)),
    Mutable(for<'a> fn(&mut Command, &'a mut World<'a>)),
}

#[derive(Debug)]
pub enum EngineCommand {
    AddSystem(System<SystemFunction>),
    SpawnEntity(EntityId, Vec<Box<dyn DynComponent>>),
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
