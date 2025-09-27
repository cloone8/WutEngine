//! Bindings for the `wutengine.math` Lua module

use wutengine_lua::mlua::{self, IntoLua, Lua, Number};
use wutengine_math::Vec4;

use crate::scripting::lua::math::vec4::LuaVec4;

pub mod vec4;

fn get_table_float(
    lua: &Lua,
    table: &mlua::Table,
    key: impl IntoLua,
    default: f64,
) -> Result<f64, mlua::Error> {
    let val = table.get::<mlua::Value>(key)?;

    if val.is_nil() {
        return Ok(default);
    }

    let type_name = val.type_name();

    let Some(num) = lua.coerce_number(val)? else {
        return Err(mlua::Error::FromLuaConversionError {
            from: type_name,
            to: "float".to_string(),
            message: None,
        });
    };

    Ok(num)
}

pub(crate) fn add_math_module() {
    wutengine_lua::add_module("wutengine.math", |lua| {
        let vec4 = lua.create_function(|_lua, args: LuaVec4| Ok(args))?;
        let splat4 =
            lua.create_function(|_lua, args: Number| Ok(LuaVec4(Vec4::splat(args as f32))))?;

        let module_table = lua.create_table()?;

        module_table.set("vec4", vec4)?;
        module_table.set("splat4", splat4)?;

        Ok(mlua::Value::Table(module_table))
    });
}
