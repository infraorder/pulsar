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
    components::{config::ConfigAsset, grid::Grid, lua::LuaAsset, nodes::lua::LuaType},
    lua::{init_instance, load_fn},
};

use super::{
    generic::GenericNode,
    lua::{init_lua, IsLuaNode, LuaHandle, LuaNode},
    types::{
        AudioNode, ChainType, Node, NodeBP, NodeData, NodeTrait, NodeType, NotSetup, Position,
    },
    util::{create_default_components, spawn_node_with_children},
};

pub fn init_temp(mut commands: Commands, asset_server: Res<AssetServer>) {
    load_node(
        &mut commands,
        &asset_server,
        NodeType::Instrument,
        "lua_pulse".to_string(),
        Position::new(0, 0),
    );

    load_node(
        &mut commands,
        &asset_server,
        NodeType::Transmitter,
        "lua_read".to_string(),
        Position::new(1, 0),
    );
}

pub fn keyboard_input(
    config: Res<ConfigAsset>,
    mut commands: Commands,
    mut g_query: Query<&mut Grid>,
    query: Query<(Entity, &mut GenericNode), (Without<NotSetup>, With<NodeBP>)>,
    asset_server: Res<AssetServer>,
    lua_assets: Res<Assets<LuaAsset>>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        info!("pressed space");
        let mut grid = g_query.single_mut();
        insert_node(
            &mut grid,
            config,
            &mut commands,
            query,
            asset_server,
            lua_assets,
            "lua_pulse".to_string(),
            Position::new(0, 0),
        );
    } else if keys.just_pressed(KeyCode::A) {
        info!("pressed a");
        let mut grid = g_query.single_mut();
        insert_node(
            &mut grid,
            config,
            &mut commands,
            query,
            asset_server,
            lua_assets,
            "lua_read".to_string(),
            Position::new(0, -5),
        );
    }
}

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

pub fn insert_node(
    grid: &mut Grid,
    config: Res<ConfigAsset>,
    commands: &mut Commands,
    query: Query<(Entity, &mut GenericNode), (Without<NotSetup>, With<NodeBP>)>,
    asset_server: Res<AssetServer>,
    lua_assets: Res<Assets<LuaAsset>>,
    name: String,
    pos: Position,
) {
    if let Some((_, gen_node)) = query.into_iter().find(|(_, node)| node.name() == name) {
        match gen_node {
            GenericNode::Lua(node) => {
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

                let (t_node, mut input_slots, mut output_slots) =
                    create_default_components(GenericNode::Lua(lnode));

                match grid.check_collision(&t_node, &input_slots, &output_slots) {
                    Ok(_) => {}
                    Err(e) => {
                        info!("Collision detected - {}", e);
                        // TODO: display this
                        return;
                    }
                }

                let entity = spawn_node_with_children(
                    grid,
                    &config,
                    commands,
                    &asset_server,
                    t_node,
                    &mut input_slots,
                    &mut output_slots,
                );

                let mut ce = commands.entity(entity);

                node_list.iter().for_each(|item| {
                    ce.insert(item.clone());
                });
            }
            GenericNode::Native(node) => {
                panic!("not implemented")
            }
        }
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
    node_type: NodeType,
    name: String,
    pos: Position,
) {
    match node_type {
        NodeType::Instrument => {
            load_node_instrument(commands, asset_server, name, pos);
        }
        NodeType::Transmitter => {
            load_rust_node_transmitter(commands, asset_server, name, pos);
        }
    }
}
