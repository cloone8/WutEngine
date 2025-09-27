#[cfg(feature = "lua")]
pub mod lua;

pub(crate) fn init_scripting_backends() {
    #[cfg(feature = "lua")]
    {
        lua::init_lua_backend();
    }
}
