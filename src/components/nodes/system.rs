use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use bevy::{
    asset::{AssetServer, Assets, Handle},
    ecs::{
        entity::Entity,
        query::{With, Without},
        system::{Commands, Query, Res},
    },
    input::{keyboard::KeyCode, Input},
    log::info,
};

use crate::{
    components::{lua::LuaAsset, config::ConfigAsset},
    lua::{init_instance, load_fn},
};

use super::{
    init_lua,
    types::{
        AudioNode, ChainType, LuaHandle, LuaNode, LuaType, Node, NodeBP, NodeData, NotSetup,
        Position,
    },
    util::{create_default_components, spawn_node_with_children},
};

pub fn init_temp(commands: Commands, asset_server: Res<AssetServer>) {
    load_node_instrument(
        commands,
        &asset_server,
        "lua_pulse".to_string(),
        Position::new(0, 0),
    );
}

pub fn keyboard_input(
    config: Res<ConfigAsset>,
    mut commands: Commands,
    query: Query<(Entity, &mut LuaNode), (Without<NotSetup>, With<NodeBP>)>,
    asset_server: Res<AssetServer>,
    lua_assets: Res<Assets<LuaAsset>>,

    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        insert_node(
            config,
            &mut commands,
            query,
            asset_server,
            lua_assets,
            "lua_pulse".to_string(),
            Position::new(0, 0),
        );
    }
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

        info!("Node initialized - {:?}", n);
        info!("Node Data initialized - {:?}", nd);

        node.node = n; // node now loaded from lua
        node.data = nd;
    });

    if !successful.load(Ordering::Relaxed) {
        return; // retry later
    }

    for (entity, n) in query.iter() {
        commands.entity(entity).remove::<NotSetup>();

        info!("Node initialized - {}", n.node.name);
    }
}

pub fn insert_node(
    config: Res<ConfigAsset>,
    commands: &mut Commands,
    query: Query<(Entity, &mut LuaNode), (Without<NotSetup>, With<NodeBP>)>,
    asset_server: Res<AssetServer>,
    lua_assets: Res<Assets<LuaAsset>>,
    name: String,
    pos: Position,
) {
    if let Some((_, node)) = query.into_iter().find(|(_, node)| node.node.name == name) {
        let mut lnode = LuaNode {
            node: node.node.clone(),
            data: node.data.clone(),
            handles: node.handles.clone(),
            lua: Mutex::new(init_instance()),
        };

        lnode.node.pos = pos;

        init_lua(&lua_assets, &mut lnode);

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

        let (t_node, mut input_slots, mut output_slots) = create_default_components(lnode);

        spawn_node_with_children(
            &config,
            commands,
            &asset_server,
            t_node,
            &mut input_slots,
            &mut output_slots,
        );
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
