//! Overriding default Lua functionality

/// Configurable overrides for the Lua scripting backend
#[derive(Default)]
pub struct LuaOverrides {
    /// Override for the output of the default `print()` function
    pub print: Option<Box<dyn LuaPrint>>,
}

/// Lua `print()` output
pub trait LuaPrint: Send + 'static {
    /// Receives the already formatted Lua `print()` output, and prints it to a destination
    /// of choice
    fn print(&self, val: &str);
}

fn lua_print_to_writer(
    printer: &dyn LuaPrint,
    _lua: &mlua::Lua,
    args: mlua::MultiValue,
) -> Result<(), mlua::Error> {
    let mut string_vals = Vec::with_capacity(args.len());

    for arg in args {
        let s = arg.to_string()?;
        string_vals.push(s);
    }

    let full_string = string_vals.join("\t");

    printer.print(&full_string);

    Ok(())
}

pub(crate) fn override_print_output(lua: &mlua::Lua, printer: Box<dyn LuaPrint>) {
    let cur_globals = lua.globals();

    cur_globals
        .set(
            "print",
            lua.create_function(move |lua, args| lua_print_to_writer(&*printer, lua, args))
                .expect("Failed to create print function"),
        )
        .expect("Failed to set print function");

    lua.set_globals(cur_globals)
        .expect("Failed to reset globals");
}
