use std::ops::{Mul, Neg};

use bevy::{
    asset::AssetServer,
    ecs::{
        entity::Entity,
        event::{EventReader, EventWriter},
        query::{With, Without},
        system::{Commands, Query, Res},
    },
    hierarchy::{BuildChildren, DespawnRecursiveExt, Parent},
    log::info,
    math::Vec2,
    transform::components::Transform,
};

use crate::components::{
    config::ConfigAsset,
    grid::Grid,
    nodes::{
        generic::util::{calculate_grid_pos, construct_pulse_node},
        types::{
            InputSlot, NodeTrait, NodeType, NodeVarient, ParentNode, Position, Pulse, SlotData,
        },
        util::{spawn_child_node_with_text, spawn_node_with_text},
    },
};

use super::{types::AudioNodeChangeEvent, GenericNode};

pub fn spawn_audio_pulses(
    mut ev_audio_node_change: EventReader<AudioNodeChangeEvent>,
    mut commands: Commands,
    config: Res<ConfigAsset>,
    mut g_query: Query<&mut Grid>,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &GenericNode)>,
) {
    if let Ok(mut grid) = g_query.get_single_mut() {
        for ev in ev_audio_node_change.read() {
            info!("audio node change event {:?}", ev.0);

            commands.get_entity(ev.0).unwrap();
            let (entity, node) = query.get(ev.0).unwrap();

            node.get_node()
                .output_slots
                .iter()
                .enumerate()
                .for_each(|(i, slot)| match slot.signal_type {
                    NodeType::SignalLink => {
                        let direction = node.get_node().output_slots[i].direction.clone();
                        let name = NodeVarient::AudioProd;
                        let display = "D".to_string();
                        let ntype = vec![NodeType::Prod];
                        let data = SlotData::Bang(true);

                        let pulse = Pulse {
                            slot_idx: i,
                            direction: direction.clone(),
                        };
                        let pos = calculate_grid_pos(node, &pulse, direction.clone());
                        info!("pos: {:?}", pos);
                        let node = construct_pulse_node(pos.clone(), name, display, ntype, data);
                        let mut child_entity = None;
                        commands.entity(entity).with_children(|cb| {
                            child_entity = Some(spawn_child_node_with_text(
                                &mut grid,
                                &config,
                                cb,
                                &asset_server,
                                node,
                                &Position::new(0, 0),
                            ));
                        });
                        commands.entity(child_entity.unwrap()).insert(pulse);
                    }
                    _ => (),
                });
        }
    }
}

pub fn tick_pulses(
    mut commands: Commands,
    mut g_query: Query<&mut Grid>,
    config: Res<ConfigAsset>,
    mut query: Query<(Entity, &Parent, &mut GenericNode, &Pulse, &mut Transform), With<Pulse>>,
    mut node_query: Query<&mut GenericNode, Without<Pulse>>,
    input_node_query: Query<(&Parent, &mut InputSlot)>,
    mut ev_audio_change: EventWriter<AudioNodeChangeEvent>,
) {
    let box_size = Vec2::new(config.grid_offset_x, config.grid_offset_y);

    if let Some(mut grid) = g_query.iter_mut().next() {
        for (entity, parent, mut gnode, pulse, mut tform) in query.iter_mut() {
            let node_data = gnode.get_data().clone();
            let node = gnode.get_node_mut();

            let new_pos = node.pos.offset(&pulse.direction);

            info!("CURRENT POS: {:?}", node.pos);
            info!("NEW POS: {:?}", new_pos);

            if new_pos.x > grid.dims.0
                || new_pos.x < -grid.dims.0
                || new_pos.y > grid.dims.1
                || new_pos.y < -grid.dims.1
            {
                info!("despawning pulse node");
                grid.remove_from_grid(node.pos.to_tuple());
                commands.entity(entity).despawn_recursive();
                ev_audio_change.send(AudioNodeChangeEvent(parent.get()));
                continue;
            }

            match grid.get_entity(new_pos.to_tuple()) {
                Some(e) => {
                    info!("found entity: {:?}", e);

                    if let Ok((parent_entity, input_slot)) = input_node_query.get(e) {
                        let idx = input_slot.idx;

                        if let Ok(mut parent_node) = node_query.get_mut(parent_entity.get()) {
                            let lnode_data = parent_node.get_data_mut();

                            lnode_data.slot_data[idx] = (&node_data.data).clone();
                            lnode_data.updated.push(idx);
                        }
                    } else {
                        ev_audio_change.send(AudioNodeChangeEvent(parent.get()));
                    }

                    grid.remove_from_grid(node.pos.to_tuple());
                    commands.entity(entity).despawn_recursive();
                }
                None => {
                    grid.move_entity(entity, node.pos.to_tuple(), new_pos.to_tuple());
                    node.pos = new_pos.clone();
                    tform.translation = new_pos.to_vec2().extend(0.).mul(box_size.extend(0.0));
                }
            }
        }
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
