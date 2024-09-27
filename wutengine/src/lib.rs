use command::{Command, OpenWindowParams};

#[doc(inline)]
pub use wutengine_core as core;

use wutengine_ecs::world::World;
use wutengine_ecs::Dynamic;

#[doc(inline)]
pub use wutengine_graphics as graphics;

use wutengine_core::{EntityId, System};

mod embedded {
    use include_dir::{include_dir, Dir};

    pub(crate) static SHADERS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/shaders");
}

pub mod builtins;
pub mod command;
pub mod ecs;
pub mod log;
pub mod macros;
pub mod math;
pub mod plugins;
pub mod renderer;
pub mod runtime;

//NOTE: This top-level module will _not_ be logged due to level filtering difficulties. Put any logic in a submodule.

#[derive(Debug)]
pub enum EngineCommand {
    AddSystem(System<World, Command>),
    SpawnEntity(Vec<Dynamic>),
    DestroyEntity(EntityId),
    OpenWindow(OpenWindowParams),
}

#[derive(Debug)]
enum WindowingEvent {
    OpenWindow(OpenWindowParams),
}
