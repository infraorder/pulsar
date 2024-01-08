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
    components::{
        lua::LuaAsset,
        nodes::{
            lua::LuaType,
            types::{ParentNode, SlotData, SlotType},
        },
    },
    lua::{init_instance, load_fn},
};

use super::{
    generic::GenericNode,
    lua::{IsLuaNode, LuaHandle},
    native::NativeNode,
    types::{ChannelType, Node, NodeBP, NodeData, NodeTrait, NotSetup, Position},
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
pub fn initialize_gen_node(
    mut commands: Commands,
    lua_assets: Res<Assets<LuaAsset>>,
    mut query: Query<(Entity, &mut GenericNode), (With<NotSetup>, With<NodeBP>, With<IsLuaNode>)>,
) {
    let successful = Arc::new(AtomicBool::new(true));

    query.par_iter_mut().for_each(|(_, mut node)| {
        successful.swap(
            match node.as_ref() {
                GenericNode::Lua(_) => {
                    initialize_node(&lua_assets, node.get_lua_node_mut().unwrap())
                }
                GenericNode::Native(_) => {
                    initialize_node(&lua_assets, node.get_native_node_mut().unwrap())
                }
            },
            Ordering::Relaxed,
        );
    });

    if !successful.load(Ordering::Relaxed) {
        return; // retry later
    }

    for (entity, n) in query.iter() {
        commands.entity(entity).remove::<NotSetup>();
        info!("Node initialized - {:?}", n.get_node().name);
    }
}

pub fn initialize_node<T: NodeTrait + ParentNode>(
    lua_assets: &Res<Assets<LuaAsset>>,
    node: &mut T,
) -> bool {
    match node.get_lua() {
        Some(lua) => {
            let lua = lua.lock().unwrap();

            let mut new_node = None;
            let mut new_data = None;

            let mut res = true;

            node.get_lua_handles().unwrap().iter().for_each(|handle| {
                if let LuaType::Node = handle.ltype {
                    let lua_asset = lua_assets.get(handle.handle.clone()); // TODO: Handle unwrap

                    match lua_asset {
                        Some(lua_asset) => {
                            load_fn(&lua, &node.get_node().name.to_string(), &lua_asset.script);
                        }
                        None => res = false,
                    }
                }
            });

            // all lua functions loaded, now we can run the reset function
            lua.context(|ctx| {
                let n = ctx.load("node").eval::<Node>();
                let nd = ctx.load("data").eval::<NodeData>();

                if n.is_err() || nd.is_err() {
                    res = false;
                }

                new_node = Some(n.unwrap());
                new_data = Some(nd.unwrap());
            });

            match res {
                true => {}
                false => return false,
            }

            drop(lua);

            *node.get_node_mut() = new_node.unwrap();
            *node.get_data_mut() = new_data.unwrap();
        }
        _ => (),
    };

    let n = node.get_node().clone();
    let data = node.get_data_mut();

    n.slots
        .clone()
        .into_iter()
        .for_each(|slot| match slot.slot_type {
            SlotType::F32 => data.slot_data.push(SlotData::F32(f32::default())),
            SlotType::I32 => data.slot_data.push(SlotData::I32(i32::default())),
            SlotType::Bang => data.slot_data.push(SlotData::Bang(bool::default())),
            SlotType::F32x2 => data
                .slot_data
                .push(SlotData::F32x2((f32::default(), f32::default()))),
            SlotType::None => data.slot_data.push(SlotData::None),
        });

    info!("Node initialized - {:?}", node.get_node());
    info!("Node Data initialized - {:?}", node.get_data());

    return true;
}

pub fn load_native_node_instrument(
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
        GenericNode::Native(NativeNode {
            node: Node {
                pos,
                ..Default::default()
            },
            data: NodeData {
                ..Default::default()
            },
            lua: Some(Mutex::new(lua)),
            handles: Some(vec![
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
            ]),
        }),
        IsLuaNode,
        NotSetup,
        NodeBP,
    ));
}

pub fn load_native_node_transmitter(
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
        GenericNode::Native(NativeNode {
            node: Node {
                pos,
                ..Default::default()
            },
            data: NodeData {
                ..Default::default()
            },
            lua: Some(Mutex::new(lua)),
            handles: Some(vec![
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
            ]),
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
            load_native_node_instrument(commands, asset_server, name, pos);
        }
        ChannelType::Transmitter => {
            load_native_node_transmitter(commands, asset_server, name, pos);
        }
    }
}
