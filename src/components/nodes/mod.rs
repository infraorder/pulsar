pub mod lua_pulse;
pub mod types;
pub mod util;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use bevy::{
    asset::{AssetServer, Assets, Handle},
    ecs::{
        entity::Entity,
        query::{With, Without},
        system::{Commands, Query, Res}, bundle::Bundle,
    },
    hierarchy::BuildChildren,
};

use crate::lua::{init_instance, load_fn};

use self::{
    types::{
        AudioNode, ChainType, Display, LuaHandle, LuaNode, LuaType, Node, NodeBP, NodeData,
        NotSetup, Position, OutputSlot, SenderSlot,
    },
    util::{spawn_e_with_c, spawn_text2d},
};

use super::lua::LuaAsset;

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
    mut query: Query<(Entity, &mut LuaNode), (With<NotSetup>, With<NodeBP>)>,
) {
    let successful = Arc::new(AtomicBool::new(true));

    query.par_iter_mut().for_each(|(_, mut node)| {
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

    if !successful.load(Ordering::Relaxed) {
        return; // retry later
    }

    for (entity, _) in query.iter() {
        commands.entity(entity).remove::<NotSetup>();
    }
}

pub fn spawn_node(
    mut commands: Commands,
    mut query: Query<(Entity, &mut LuaNode), (Without<NotSetup>, With<NodeBP>)>,
    asset_server: Res<AssetServer>,
    lua_assets: Res<Assets<LuaAsset>>,
    name: String,
    pos: Position,
) {
    if let Some((e, node)) = query.into_iter().find(|(e, node)| node.node.name == name) {
        let mut lnode = LuaNode {
            node: node.node.clone(),
            data: node.data.clone(),
            handles: node.handles.clone(),
            lua: Mutex::new(init_instance()),
        };
        init_lua(&lua_assets, &mut lnode);

        // lnode.node.output_slots

        let mut node_list = vec![];

        if node
            .node
            .output_slots
            .iter()
            .find(|x| match x.signal_type {
                ChainType::SignalConst => true,
                _ => false,
            })
            .is_some()
        {
            node_list.push(AudioNode);
        }

        let t_bundle = spawn_e_with_c(&asset_server, &lnode);

        let mut e = commands
            .spawn((lnode, t_bundle))
            .with_children(|builder| {});

        node_list.iter().for_each(|x| {
            e.insert(x.to_owned());
        });
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
        NodeBP,
    ));
}

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

pub fn create_components(c: LuaNode) -> (LuaNode, Vec<OutputSlot>, Vec<SenderSlot>) {
    let mut t = Vec::new();
    let mut s = Vec::new();

    c.node.output_slots.iter().enumerate().for_each(|i, o_s| {
        t.push(OutputSlot {

            idx: i,

        });
    });

    (c, t, s)
}
