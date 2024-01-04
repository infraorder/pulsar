use std::f64::consts::{PI, TAU};

use rlua::Lua;

pub const UTIL_STRING: &str = include_str!("../../assets/lua/util.lua");

pub fn init_instance() -> Lua {
    let lua = Lua::new();

    load_globals(&lua);
    load_fn(&lua, "lua_pulse", UTIL_STRING);

    lua
}

pub fn load_globals(lua: &Lua) {
    lua.context(|lua_ctx| {
        let globals = lua_ctx.globals();

        globals.set("pi", PI).unwrap();
        globals.set("tau", TAU).unwrap();
        // TODO: Add more globals here
    });
}

pub fn load_fn(lua: &Lua, name: &str, method_name: &str) {
    lua.context(|lua_ctx| {
        let _ = lua_ctx.load(method_name).set_name(name).unwrap().exec();
    });
}
