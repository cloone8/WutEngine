//! Lua scripting for WutEngine

use mlua::{AsChunk, IntoLuaMulti, LuaOptions, StdLib};
use wutengine_util::GlobalManager;

use crate::overrides::LuaOverrides;

pub mod overrides;

pub use mlua;

#[derive(Debug)]
pub(crate) struct LuaManager {
    lua: mlua::Lua,
}

pub(crate) static LUA_MANAGER: GlobalManager<LuaManager> = GlobalManager::new();

/// Initializes the global [LuaManager]
#[doc(hidden)]
pub fn init(custom_overrides: LuaOverrides) {
    GlobalManager::init(&LUA_MANAGER, LuaManager::new());

    if let Some(print_override) = custom_overrides.print {
        overrides::override_print_output(&LUA_MANAGER.lua, print_override);
    }
}

/// Adds a module to the Lua scripting environment
pub fn add_module(
    name: &str,
    provider: impl Fn(&mlua::Lua) -> Result<mlua::Value, mlua::Error> + Send + 'static,
) {
    let name_string = name.to_string();

    let loader = LUA_MANAGER
        .lua
        .create_function(move |lua, args: mlua::MultiValue| {
            let mod_to_load = args[0].as_string().expect("Tried to load non-string");

            assert_eq!(
                mod_to_load.to_str().expect("Required non-utf8 string"),
                name_string
            );

            provider(lua)
        })
        .expect("Could not set loader");

    LUA_MANAGER.lua.preload_module(name, loader).unwrap();
}

impl LuaManager {
    fn get_default_stdlibs() -> StdLib {
        StdLib::TABLE | StdLib::IO | StdLib::OS | StdLib::STRING | StdLib::MATH | StdLib::PACKAGE
    }
    fn new() -> Self {
        Self {
            lua: mlua::Lua::new_with(Self::get_default_stdlibs(), LuaOptions::new())
                .expect("Failed to initialize Lua"),
        }
    }
}

/// Shim for [run_script_with_args<R>(())] with zero arguments
pub fn run_script<R: mlua::FromLuaMulti>(script: impl AsChunk) -> Result<R, mlua::Error> {
    run_script_with_args::<R>(script, ())
}

/// Runs a script with the given arguments, converting the scripts return values to the type `R`
pub fn run_script_with_args<R: mlua::FromLuaMulti>(
    script: impl AsChunk,
    args: impl IntoLuaMulti,
) -> Result<R, mlua::Error> {
    let loaded = LUA_MANAGER.lua.load(script);

    loaded.call::<R>(args)
}
