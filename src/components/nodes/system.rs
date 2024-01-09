use std::sync::Mutex;

use bevy::{
    asset::{AssetServer, Assets},
    ecs::{
        entity::Entity,
        event::EventWriter,
        query::{With, Without},
        system::{Commands, Query, Res, ResMut},
    },
    input::{keyboard::KeyCode, Input},
    log::info,
};
use knyst::{
    controller::KnystCommands,
    graph::{connection::InputBundle, GenOrGraph, Graph, GraphSettings},
    inputs, knyst_commands,
};

use crate::{
    components::{
        config::ConfigAsset,
        grid::Grid,
        lua::LuaAsset,
        nodes::{lua::get_lua_wave_handles, types::NodeVarient},
    },
    dsp::{oscillators::Oscillator, Dsp},
    lua::init_instance,
};

use super::{
    generic::{types::AudioNodePulseEvent, GenericNode},
    lua::{init_lua, LuaNode},
    native::NativeNode,
    types::{AudioNode, NodeBP, NodeTrait, NodeType, NotSetup, ParentNode, Position, Slot},
    util::{create_default_components, spawn_node_with_children},
};

pub fn keyboard_input_temp(
    config: Res<ConfigAsset>,
    mut commands: Commands,
    mut g_query: Query<&mut Grid>,
    query: Query<(Entity, &mut GenericNode), (Without<NotSetup>, With<NodeBP>)>,
    asset_server: Res<AssetServer>,
    lua_assets: Res<Assets<LuaAsset>>,
    keys: Res<Input<KeyCode>>,
    ev_audio_change: EventWriter<AudioNodePulseEvent>,
) {
    if keys.just_pressed(KeyCode::Space) {
        info!("pressed SPACE");
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
            ev_audio_change,
        );
    } else if keys.just_pressed(KeyCode::A) {
        info!("pressed A");
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
            ev_audio_change,
        );
    } else if keys.just_pressed(KeyCode::R) {
        info!("pressed R");
        let mut grid = g_query.single_mut();
        insert_node(
            &mut grid,
            config,
            &mut commands,
            query,
            asset_server,
            lua_assets,
            "audio_out".to_string(),
            Position::new(0, -10),
            ev_audio_change,
        );
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
    mut ev_audio_change: EventWriter<AudioNodePulseEvent>,
) {
    if let Some((_, gen_node)) = query
        .into_iter()
        .find(|(_, node)| node.name().to_string() == name)
    {
        match gen_node {
            GenericNode::Lua(node) => {
                let mut lnode = construct_lua_node_from_node_bp(node, pos);
                init_lua(&lua_assets, &mut lnode);

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

                // audio not yet supported for fully lua based nodes

                info!("spawned node: {:?}", entity);
            }
            GenericNode::Native(node) => {
                let mut lnode = construct_native_node_from_node_bp(node, pos);
                init_lua(&lua_assets, &mut lnode);

                let mut node_list = vec![];

                let (t_node, mut input_slots, mut output_slots) =
                    create_default_components(GenericNode::Native(lnode));

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

                let slots = contains_audio(node);
                slots.iter().for_each(|(i, slot)| {
                    info!("contains audio");

                    let mut last_idx = 0;

                    match node.name() {
                        NodeVarient::LuaPulse => {
                            info!("inserting pulse");

                            let osc = Oscillator {
                                lua_handle: get_lua_wave_handles(node),
                                lua_string: "".to_string(),
                            };

                            let k = knyst_commands().push(
                                Graph::new(GraphSettings {
                                    block_size: 64,
                                    ..Default::default()
                                }),
                                inputs!(),
                            );

                            let l = graph.get_chain_mut();
                            l.push(TChain::vec(
                                vec![TChain::dsp(Dsp::Input(osc), Some(entity))],
                                Some(entity),
                            ));
                            last_idx = l.len() - 1;
                        }
                        NodeVarient::AudioOut => {
                            info!("inserting audio out");

                            let l = graph.get_chain_mut();
                            l.push(TChain::vec(
                                vec![TChain::dsp(Dsp::Output, Some(entity))],
                                Some(entity),
                            ));
                            last_idx = l.len() - 1;
                        }
                        _ => (),
                    }

                    node_list.push(AudioNode {
                        idx: Some(last_idx),
                    });
                    ev_audio_change.send(AudioNodePulseEvent {
                        entity,
                        slot_idx: i.to_owned(),
                    });
                });

                if slots.len() > 0 {
                    let mut ce = commands.entity(entity);

                    node_list.iter().for_each(|item| {
                        ce.insert(item.clone());
                    });
                }

                info!("spawned node: {:?}", entity);
            }
        }
    }
}

fn contains_audio<T: ParentNode>(node: &T) -> Vec<(usize, &Slot)> {
    node.get_node()
        .output_slots
        .iter()
        .enumerate()
        .filter(|(i, x)| match x.signal_type {
            NodeType::SignalLink => true,
            _ => false,
        })
        .collect()
}

fn construct_lua_node_from_node_bp(node: &LuaNode, pos: Position) -> LuaNode {
    let mut sub_node = node.node.clone();
    sub_node.pos = pos;

    let lnode = LuaNode {
        node: sub_node,
        data: node.data.clone(),
        handles: node.handles.clone(),
        lua: Mutex::new(init_instance()),
    };

    lnode
}

fn construct_native_node_from_node_bp(node: &NativeNode, pos: Position) -> NativeNode {
    let mut sub_node = node.node.clone();
    sub_node.pos = pos;

    let lnode = NativeNode {
        node: sub_node,
        data: node.data.clone(),
        handles: node.handles.clone(),
        lua: Some(Mutex::new(init_instance())),
    };

    lnode
}
