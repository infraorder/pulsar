use std::sync::Mutex;

use bevy::{
    asset::{AssetServer, Assets, Handle},
    ecs::{
        entity::Entity,
        query::With,
        system::{Commands, Query, Res},
    },
};

use crate::lua::{init_instance, load_fn};

use self::types::{
    ChainType, LuaHandle, LuaNode, LuaType, Node, NodeData, NotSetup, Position, Silent,
};

use super::lua::LuaAsset;

pub mod lua_pulse;
pub mod types;

pub fn init_temp(commands: Commands, asset_server: Res<AssetServer>) {
    load_node_instrument(
        commands,
        &asset_server,
        "lua_pulse".to_string(),
        Position::new(0, 0),
    );
}

pub fn initialize_node(
    mut commands: Commands,
    lua_assets: Res<Assets<LuaAsset>>,
    mut query: Query<(Entity, &mut LuaNode), With<NotSetup>>,
) {
    query.par_iter_mut().for_each(|(_, mut node)| {
        let lua = node.lua.lock().unwrap();
        node.handles.iter().for_each(|handle| {
            if let LuaType::Node = handle.ltype {
                let lua_asset = lua_assets.get(handle.handle.clone()).unwrap(); // TODO: Handle unwrap
                load_fn(
                    &lua,
                    &node.node.name,
                    &lua_assets.get(handle.handle.clone()).unwrap().script,
                );
            }
        });
        let mut n = node.node.clone();
        let mut nd = node.data.clone();
        // all lua functions loaded, now we can run the reset function
        lua.context(|ctx| {
            n = ctx.load("node").eval::<Node>().unwrap();
            nd = ctx.load("data").eval::<NodeData>().unwrap();
        });
        drop(lua);
        node.node = n; // node now loaded from lua
        node.data = nd;
    });

    for (entity, node) in query.iter() {
        if node.node.output_slots.iter().any(|x| {
            if let ChainType::SignalConst = x.signal_type {
                true
            } else {
                false
            }
        }) {

            // this is an audio node
        }

        commands.entity(entity).remove::<NotSetup>();
    }
}

pub fn load_node_instrument(
    mut commands: Commands,
    asset_server: &Res<AssetServer>,
    name: String,
    pos: Position,
) {
    // base logic
    let base_node_handle: Handle<LuaAsset> = asset_server.load("lua/common/node.lua");
    let base_instrument_node_handle: Handle<LuaAsset> =
        asset_server.load("lua/common/instrument/node.lua");
    let base_instrument_wave_handle: Handle<LuaAsset> =
        asset_server.load("lua/common/instrument/wave.lua");

    // custom logic
    let lua_node_handle: Handle<LuaAsset> =
        asset_server.load(format!("lua/nodes/instrument/{}/node.lua", name));
    let lua_wave_handle: Handle<LuaAsset> =
        asset_server.load(format!("lua/nodes/instrument/{}/wave.lua", name));

    let lua = init_instance();

    commands.spawn((
        LuaNode {
            node: Node {
                pos,
                ..Default::default()
            },
            data: NodeData {
                ..Default::default()
            },
            lua: Mutex::new(lua),
            handles: vec![
                LuaHandle {
                    ltype: LuaType::Node,
                    handle: base_node_handle,
                },
                LuaHandle {
                    ltype: LuaType::Node,
                    handle: base_instrument_node_handle,
                },
                LuaHandle {
                    ltype: LuaType::Node,
                    handle: lua_node_handle,
                },
                LuaHandle {
                    ltype: LuaType::Wave,
                    handle: base_instrument_wave_handle,
                },
                LuaHandle {
                    ltype: LuaType::Wave,
                    handle: lua_wave_handle,
                },
            ],
        },
        NotSetup,
    ));
}
