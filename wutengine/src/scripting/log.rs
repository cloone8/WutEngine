use wutengine_lua::mlua;

pub(super) fn add_log_module() {
    wutengine_lua::add_module("wutengine.log", |lua| {
        let trace =
            lua.create_function(|lua, args| log_lua_with_level(lua, args, log::Level::Trace))?;
        let debug =
            lua.create_function(|lua, args| log_lua_with_level(lua, args, log::Level::Debug))?;
        let info =
            lua.create_function(|lua, args| log_lua_with_level(lua, args, log::Level::Info))?;
        let warn =
            lua.create_function(|lua, args| log_lua_with_level(lua, args, log::Level::Warn))?;
        let err =
            lua.create_function(|lua, args| log_lua_with_level(lua, args, log::Level::Error))?;

        let module_table = lua.create_table()?;

        module_table.set("trace", trace)?;
        module_table.set("debug", debug)?;
        module_table.set("info", info)?;
        module_table.set("warn", warn)?;
        module_table.set("error", err)?;

        Ok(mlua::Value::Table(module_table))
    });
}

fn log_lua_with_level(
    lua: &mlua::Lua,
    args: mlua::MultiValue,
    level: log::Level,
) -> Result<(), mlua::Error> {
    let Some((file, line, message)) = lua.inspect_stack(1, |dbg| {
        (
            dbg.source().short_src.map(|src| src.to_string()),
            dbg.current_line(),
            args.into_iter()
                .map(|val| val.to_string().unwrap())
                .collect::<Vec<_>>()
                .join("\t"),
        )
    }) else {
        panic!("Failed to inspect log stack");
    };

    let message = format_args!("{}", message);
    let file = file.unwrap_or("(unknown file)".to_string());

    let record = log::RecordBuilder::new()
        .level(level)
        .target("lua_script")
        .file(Some(&file))
        .line(line.map(|line| line as u32))
        .args(message)
        .build();

    log::logger().log(&record);

    Ok(())
}
