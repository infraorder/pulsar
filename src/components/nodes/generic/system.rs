use std::{ops::Mul, process::Output};

use anyhow::Chain;
use bevy::{
    asset::AssetServer,
    audio,
    ecs::{
        entity::Entity,
        event::{EventReader, EventWriter},
        query::{With, Without},
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::{DespawnRecursiveExt, Parent},
    log::info,
    math::Vec2,
    transform::components::Transform,
};
use knyst::knyst_commands;

use crate::{
    components::{
        audio::AudioGraph,
        config::ConfigAsset,
        grid::Grid,
        nodes::{
            generic::util::{calculate_grid_pos, construct_pulse_node},
            lua::get_lua_wave_handles,
            types::{AudioNode, InputSlot, NodeType, NodeVarient, ParentNode, Pulse, SlotData},
            util::spawn_node_with_text,
        },
    },
    dsp::{oscillators::Oscillator, read::Read, ChainType, Dsp, TChain},
};

use super::{types::AudioNodePulseEvent, GenericNode};

pub fn spawn_audio_pulses(
    mut ev_audio_pulse_event: EventReader<AudioNodePulseEvent>,
    mut commands: Commands,
    config: Res<ConfigAsset>,
    mut g_query: Query<&mut Grid>,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &GenericNode)>,
) {
    if let Ok(mut grid) = g_query.get_single_mut() {
        for ev in ev_audio_pulse_event.read() {
            info!("audio node change event {:?}", ev.entity);

            commands.get_entity(ev.entity).unwrap();
            let (entity, node) = query.get(ev.entity).unwrap();

            let slot = &node.get_node().output_slots[ev.slot_idx];

            match slot.signal_type {
                NodeType::SignalLink => {
                    let direction = node.get_node().output_slots[ev.slot_idx].direction.clone();
                    let name = NodeVarient::AudioProd;
                    let display = "D".to_string();
                    let ntype = vec![NodeType::Prod];
                    let data = SlotData::Bang(true);

                    let pulse = Pulse {
                        slot_idx: ev.slot_idx,
                        direction: direction.clone(),
                        original_entity: entity,
                    };
                    let pos = calculate_grid_pos(node, &pulse, direction.clone());

                    info!("pos: {:?}", pos);
                    let node = construct_pulse_node(pos.clone(), name, display, ntype, data);
                    let e = spawn_node_with_text(
                        &mut grid,
                        &config,
                        &mut commands,
                        &asset_server,
                        node,
                    );
                    commands.entity(e).insert(pulse);
                }
                _ => (),
            }
        }
    }
}

// main scheduled system for pulses.
pub fn tick_pulses(
    mut commands: Commands,
    mut g_query: Query<&mut Grid>,
    config: Res<ConfigAsset>,
    mut graph: ResMut<AudioGraph>,
    mut query: Query<(Entity, &mut GenericNode, &Pulse, &mut Transform), With<Pulse>>,
    mut node_query: Query<(&mut GenericNode), Without<Pulse>>,
    mut audio_node_query: Query<&AudioNode>,
    input_node_query: Query<(&Parent, &mut InputSlot)>,
    mut ev_audio_pulse: EventWriter<AudioNodePulseEvent>,
) {
    // box size config.
    let box_size = Vec2::new(config.grid_offset_x, config.grid_offset_y);

    // get the grid
    if let Some(mut grid) = g_query.iter_mut().next() {
        // iterate over all pulses
        for (entity, mut gnode, pulse, mut tform) in query.iter_mut() {
            let node_data = gnode.get_data().clone();
            let node = gnode.get_node_mut();

            let current_pos = node.pos;
            let new_pos = current_pos.offset(&pulse.direction);

            info!("-----------------------------------");
            info!("running pulse move on pulse entity: {:?}", entity);
            info!("pulse move - current position: {:?}", current_pos);
            info!("pulse move - new position: {:?}", new_pos);

            // check if the pulse is out of bounds.
            if new_pos.x > grid.dims.0
                || new_pos.x < -grid.dims.0
                || new_pos.y > grid.dims.1
                || new_pos.y < -grid.dims.1
            {
                info!("despawning pulse node from grid and resending audio pulse event");

                grid.remove_from_grid(current_pos.to_tuple());
                commands.entity(entity).despawn_recursive();
                ev_audio_pulse.send(AudioNodePulseEvent {
                    entity: pulse.original_entity,
                    slot_idx: pulse.slot_idx,
                });
                continue;
            }

            // check if the pulses new position is colliding with another node.
            match grid.get_entity(new_pos.to_tuple()) {
                Some(e) => {
                    info!("found entity in grid: {:?}", e);

                    // check if the node is an input node. - ignore the rest but despawn the pulse.
                    if let Ok((parent_entity, input_slot)) = input_node_query.get(e) {
                        let idx = input_slot.idx;

                        // get root node of input.
                        if let Ok(gnode) = node_query.get_mut(parent_entity.get()) {
                            // check if the node is a signal const node, and if the signal type is audio.
                            match (&gnode.get_node().slots[idx].signal_type, &node.name) {
                                (NodeType::SignalConst, NodeVarient::AudioProd) => {
                                    connect_audio(
                                        &audio_node_query,
                                        pulse,
                                        &mut graph,
                                        gnode,
                                        entity,
                                    );
                                }
                                _ => (),
                            }
                        }
                    } else {
                        info!("sending audio pulse event.");
                        ev_audio_pulse.send(AudioNodePulseEvent {
                            entity: pulse.original_entity,
                            slot_idx: pulse.slot_idx,
                        })
                    }

                    // remove pulse from grid and despawn.
                    grid.remove_from_grid(current_pos.to_tuple());
                    commands.entity(entity).despawn_recursive();
                }
                None => {
                    // move pulse to new position.
                    grid.move_entity(entity, current_pos.to_tuple(), new_pos.to_tuple());
                    tform.translation = new_pos.to_vec2().extend(0.).mul(box_size.extend(0.0));
                    node.pos = new_pos;
                }
            }
        }
    }
}

// this function is used for adding to the audio graph.
// does not deal with any data, just linking to the ast.
// TODO: need to draw some kind of line showing the connection
fn connect_audio(
    audio_node_query: &Query<'_, '_, &AudioNode>,
    pulse: &Pulse,
    graph: &mut ResMut<'_, AudioGraph>,
    gnode: bevy::prelude::Mut<'_, GenericNode>,
    entity: Entity,
) {
    // get audio node.
    let audio_node = audio_node_query.get(pulse.original_entity).unwrap();

    // check if the chain is already setup.
    match graph.get_chain_mut()[audio_node.idx.unwrap()].t.as_mut() {
        ChainType::ChainList(ref mut l) => match gnode.get_node().name {
            NodeVarient::LuaRead => {
                info!("inserting pulse");

                let osc = Read;
                l.push(TChain::vec(
                    vec![TChain::dsp(Dsp::Read(osc), Some(entity))],
                    Some(entity),
                ));
            }
            NodeVarient::AudioProd => {
                info!("inserting pulse");
                l.push(TChain::vec(
                    vec![TChain::dsp(Dsp::Output, Some(entity))],
                    Some(entity),
                ));
            }
            NodeVarient::LuaPulse => panic!("cannot lua pulse to an existing chain."),
            _ => panic!("not recognized sound node."),
        },
        // expected chain type here. creating a new chain.
        _ => {}
    }
}

pub fn tick_logic(
    mut commands: Commands,
    mut g_query: Query<&mut Grid>,
    config: Res<ConfigAsset>,
    mut query: Query<(Entity, &mut GenericNode, &mut Transform)>,
    input_node_query: Query<(&Parent, &mut InputSlot)>,
) {
    let box_size = Vec2::new(config.grid_offset_x, config.grid_offset_y);

    if let Some(mut grid) = g_query.iter_mut().next() {
        query.par_iter_mut().for_each(|(e, mut gn, tform)| {
            // Update node-data where necessary (for now just lua nodes)
            match gn.as_ref() {
                GenericNode::Lua(_) => {
                    let lua_node = gn.get_lua_node_mut().unwrap();

                    let data = lua_node.data.clone();
                    let lua = lua_node.lua.lock().unwrap();

                    lua.context(|ctx| {
                        ctx.globals().set("data", data).unwrap(); // TODO: handle error
                    });
                }
                GenericNode::Native(_) => {
                    let gen_node = gn.get_native_node_mut().unwrap();

                    // match gen_node.node.name {
                    //
                    //     // NodeVarient::LuaRead => {
                    //     //
                    //     // }
                    //
                    //
                    //
                    // }

                    let data = gen_node.data.clone();
                }
            }
        });
    }
}
