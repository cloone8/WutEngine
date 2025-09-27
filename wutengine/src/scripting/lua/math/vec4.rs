//! Implementation of the WutEngine Lua scripting API `Vec4` type

use wutengine_lua::mlua::{self, FromLua, FromLuaMulti, Lua, MetaMethod, Number};
use wutengine_math::Vec4;

use crate::scripting::lua::math::get_table_float;

/// A Lua `Vec4`
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct LuaVec4(pub(crate) Vec4);

impl LuaVec4 {
    fn from_table(lua: &mlua::Lua, table: &mlua::Table) -> Result<Self, mlua::Error> {
        let x = get_table_float(lua, table, "x", 0.0)? as f32;
        let y = get_table_float(lua, table, "y", 0.0)? as f32;
        let z = get_table_float(lua, table, "z", 0.0)? as f32;
        let w = get_table_float(lua, table, "w", 0.0)? as f32;

        Ok(Self(wutengine_math::vec4(x, y, z, w)))
    }
}

impl From<Vec4> for LuaVec4 {
    #[inline(always)]
    fn from(value: Vec4) -> Self {
        Self(value)
    }
}

impl From<LuaVec4> for Vec4 {
    #[inline(always)]
    fn from(value: LuaVec4) -> Self {
        value.0
    }
}

impl mlua::UserData for LuaVec4 {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |_lua, this| Ok(this.0.x));
        fields.add_field_method_get("y", |_lua, this| Ok(this.0.y));
        fields.add_field_method_get("z", |_lua, this| Ok(this.0.z));
        fields.add_field_method_get("w", |_lua, this| Ok(this.0.w));
    }

    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_function(MetaMethod::Add, |lua, mut args: mlua::MultiValue| {
            assert_eq!(2, args.len(), "Unexpected number of Lua args");

            let left = LuaVec4AddSubArg::from_lua(args.pop_front().unwrap(), lua)?;
            let right = LuaVec4AddSubArg::from_lua(args.pop_front().unwrap(), lua)?;

            match (left, right) {
                (LuaVec4AddSubArg::LuaVec4(v4_left), LuaVec4AddSubArg::LuaVec4(v4_right)) => {
                    Ok(Self(v4_left.0 + v4_right.0))
                }
                (LuaVec4AddSubArg::LuaVec4(v4_left), LuaVec4AddSubArg::Number(num)) => {
                    Ok(Self(v4_left.0 + Vec4::splat(num as f32)))
                }
                (LuaVec4AddSubArg::Number(num), LuaVec4AddSubArg::LuaVec4(v4_right)) => {
                    Ok(Self(Vec4::splat(num as f32) + v4_right.0))
                }
                (LuaVec4AddSubArg::Number(_), LuaVec4AddSubArg::Number(_)) => {
                    unreachable!("Cannot add two numbers as vec4s")
                }
            }
        });

        methods.add_meta_function(MetaMethod::Sub, |lua, mut args: mlua::MultiValue| {
            assert_eq!(2, args.len(), "Unexpected number of Lua args");

            let left = LuaVec4AddSubArg::from_lua(args.pop_front().unwrap(), lua)?;
            let right = LuaVec4AddSubArg::from_lua(args.pop_front().unwrap(), lua)?;

            match (left, right) {
                (LuaVec4AddSubArg::LuaVec4(v4_left), LuaVec4AddSubArg::LuaVec4(v4_right)) => {
                    Ok(Self(v4_left.0 - v4_right.0))
                }
                (LuaVec4AddSubArg::LuaVec4(v4_left), LuaVec4AddSubArg::Number(num)) => {
                    Ok(Self(v4_left.0 - Vec4::splat(num as f32)))
                }
                (LuaVec4AddSubArg::Number(num), LuaVec4AddSubArg::LuaVec4(v4_right)) => {
                    Ok(Self(Vec4::splat(num as f32) - v4_right.0))
                }
                (LuaVec4AddSubArg::Number(_), LuaVec4AddSubArg::Number(_)) => {
                    unreachable!("Cannot subtract two numbers as vec4s")
                }
            }
        });

        methods.add_meta_method(MetaMethod::ToString, |_lua, this, _args: ()| {
            Ok(format!(
                "vec4({}, {}, {}, {})",
                this.0.x, this.0.y, this.0.z, this.0.w
            ))
        });
    }
}

impl FromLuaMulti for LuaVec4 {
    fn from_lua_multi(values: mlua::MultiValue, lua: &Lua) -> mlua::Result<Self> {
        if values.is_empty() {
            return Ok(Self(Vec4::ZERO));
        }

        if let Some(as_self) = values[0]
            .as_userdata()
            .and_then(|udata| udata.borrow::<Self>().ok())
        {
            return Ok(*as_self);
        }

        // If we've been given a table, use that and ignore other arguments
        if let Some(table) = values[0].as_table() {
            return Self::from_table(lua, table);
        }

        // If no table, take the first 4 arguments as x/y/z/w. Any missing argument is set to 0.0. Extra arguments are ignored

        let x_value = values.get(0).unwrap();

        let x = lua.coerce_number(x_value.clone())?.ok_or_else(move || {
            mlua::Error::FromLuaConversionError {
                from: x_value.type_name(),
                to: "float".to_string(),
                message: None,
            }
        })?;

        let y = match values.get(1) {
            Some(y_value) => lua.coerce_number(y_value.clone())?.ok_or_else(move || {
                mlua::Error::FromLuaConversionError {
                    from: y_value.type_name(),
                    to: "float".to_string(),
                    message: None,
                }
            })?,
            None => 0.0,
        };

        let z = match values.get(2) {
            Some(z_value) => lua.coerce_number(z_value.clone())?.ok_or_else(move || {
                mlua::Error::FromLuaConversionError {
                    from: z_value.type_name(),
                    to: "float".to_string(),
                    message: None,
                }
            })?,
            None => 0.0,
        };

        let w = match values.get(3) {
            Some(w_value) => lua.coerce_number(w_value.clone())?.ok_or_else(move || {
                mlua::Error::FromLuaConversionError {
                    from: w_value.type_name(),
                    to: "float".to_string(),
                    message: None,
                }
            })?,
            None => 0.0,
        };

        Ok(Self(wutengine_math::vec4(
            x as f32, y as f32, z as f32, w as f32,
        )))
    }
}

#[derive(Debug, Clone, Copy)]
enum LuaVec4AddSubArg {
    LuaVec4(LuaVec4),
    Number(Number),
}

impl FromLua for LuaVec4AddSubArg {
    fn from_lua(value: mlua::Value, lua: &Lua) -> mlua::Result<Self> {
        match Number::from_lua(value.clone(), lua) {
            Ok(num) => Ok(Self::Number(num)),
            Err(_) => Ok(Self::LuaVec4(LuaVec4::from_lua_multi(
                vec![value].into(),
                lua,
            )?)),
        }
    }
}

//TODO: Test conversions and operations
