use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use bevy::{
    asset::{AssetServer, Assets, Handle},
    ecs::{
        entity::Entity,
        query::With,
        system::{Commands, Query, Res},
    },
    log::info,
};

use crate::{
    components::{lua::LuaAsset, nodes::lua::LuaType},
    lua::{init_instance, load_fn},
};

use super::{
    generic::GenericNode,
    lua::{IsLuaNode, LuaHandle, LuaNode},
    types::{ChannelType, Node, NodeBP, NodeData, NotSetup, Position},
};

pub fn init_temp_blueprints(mut commands: Commands, asset_server: Res<AssetServer>) {
    load_node(
        &mut commands,
        &asset_server,
        ChannelType::Instrument,
        "lua_pulse".to_string(),
        Position::new(0, 0),
    );

    load_node(
        &mut commands,
        &asset_server,
        ChannelType::Transmitter,
        "lua_read".to_string(),
        Position::new(1, 0),
    );
}

// TODO: cleanup
pub fn initialize_node(
    mut commands: Commands,
    lua_assets: Res<Assets<LuaAsset>>,
    mut query: Query<(Entity, &mut GenericNode), (With<NotSetup>, With<NodeBP>, With<IsLuaNode>)>,
) {
    let successful = Arc::new(AtomicBool::new(true));

    query.par_iter_mut().for_each(|(_, mut node)| {
        let node = node.get_lua_node_mut().unwrap();
        let lua = node.lua.lock().unwrap();
        node.handles.iter().for_each(|handle| {
            if let LuaType::Node = handle.ltype {
                let lua_asset = lua_assets.get(handle.handle.clone()); // TODO: Handle unwrap

                match lua_asset {
                    Some(lua_asset) => {
                        load_fn(&lua, &node.node.name, &lua_asset.script);
                    }
                    None => {
                        successful.swap(false, Ordering::Relaxed);
                    }
                }
            }
        });
        // all lua functions loaded, now we can run the reset function
        lua.context(|ctx| {
            node.node = ctx.load("node").eval::<Node>().unwrap();
            node.data = ctx.load("data").eval::<NodeData>().unwrap();
        });

        drop(lua);

        info!("Node initialized - {:?}", node.node);
        info!("Node Data initialized - {:?}", node.data);
    });

    if !successful.load(Ordering::Relaxed) {
        return; // retry later
    }

    for (entity, n) in query.iter() {
        let n = n.get_lua_node().unwrap();

        commands.entity(entity).remove::<NotSetup>();

        info!("Node initialized - {}", n.node.name);
    }
}

pub fn load_node_instrument(
    commands: &mut Commands,
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
        GenericNode::Lua(LuaNode {
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
        }),
        IsLuaNode,
        NotSetup,
        NodeBP,
    ));
}

pub fn load_rust_node_transmitter(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    name: String,
    pos: Position,
) {
    // base logic
    let base_node_handle: Handle<LuaAsset> = asset_server.load("lua/common/node.lua");
    let base_instrument_node_handle: Handle<LuaAsset> =
        asset_server.load("lua/common/transmitter/node.lua");

    // custom logic
    let lua_node_handle: Handle<LuaAsset> =
        asset_server.load(format!("lua/nodes/transmitter/{}/node.lua", name));

    let lua = init_instance();

    commands.spawn((
        GenericNode::Lua(LuaNode {
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
            ],
        }),
        IsLuaNode,
        NotSetup,
        NodeBP,
    ));
}

pub fn load_node(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    node_type: ChannelType,
    name: String,
    pos: Position,
) {
    match node_type {
        ChannelType::Instrument => {
            load_node_instrument(commands, asset_server, name, pos);
        }
        ChannelType::Transmitter => {
            load_rust_node_transmitter(commands, asset_server, name, pos);
        }
    }
}
