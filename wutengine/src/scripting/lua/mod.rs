//! Implementation of the WutEngine Lua scripting API

use wutengine_lua::overrides::LuaPrint;

mod log;
pub mod math;

struct LogPrinter;

impl LuaPrint for LogPrinter {
    fn print(&self, val: &str) {
        println!("{}", val);
    }
}

pub(crate) fn init_lua_backend() {
    let overrides = wutengine_lua::overrides::LuaOverrides {
        print: Some(Box::new(LogPrinter)),
    };

    wutengine_lua::init(overrides);
    log::add_log_module();
    math::add_math_module();
}
