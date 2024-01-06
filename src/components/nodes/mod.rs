pub mod system;
pub mod types;
pub mod util;

use bevy::{asset::Assets, ecs::system::Res};

use crate::lua::load_fn;

use self::types::{LuaNode, LuaType};

use super::lua::LuaAsset;

fn init_lua(lua_assets: &Res<Assets<LuaAsset>>, node: &mut LuaNode) {
    let lua = node.lua.lock().unwrap();
    node.handles.iter().for_each(|handle| {
        if let LuaType::Node = handle.ltype {
            let lua_asset = lua_assets.get(handle.handle.clone()); // TODO: Handle unwrap

            match lua_asset {
                Some(lua_asset) => {
                    load_fn(&lua, &node.node.name, &lua_asset.script);
                }
                None => panic!("this should never happen"),
            }
        }
    });
    let n = node.node.clone();
    let nd = node.data.clone();

    node.node = n;
    node.data = nd;
}
