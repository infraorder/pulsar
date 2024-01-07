use std::sync::Mutex;

use bevy::{
    asset::{AssetServer, Assets},
    ecs::{
        entity::Entity,
        event::EventWriter,
        query::{With, Without},
        system::{Commands, Query, Res},
    },
    input::{keyboard::KeyCode, Input},
    log::info,
};

use crate::{
    components::{config::ConfigAsset, grid::Grid, lua::LuaAsset},
    lua::init_instance,
};

use super::{
    generic::{GenericNode, types::AudioNodeChangeEvent},
    lua::{init_lua, LuaNode},
    types::{AudioNode, NodeBP, NodeTrait, NodeType, NotSetup, Position},
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
    ev_audio_change: EventWriter<AudioNodeChangeEvent>,
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
            ev_audio_change,
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
    mut ev_audio_change: EventWriter<AudioNodeChangeEvent>,
) {
    if let Some((_, gen_node)) = query.into_iter().find(|(_, node)| node.name() == name) {
        match gen_node {
            GenericNode::Lua(node) => {
                let mut lnode = construct_node_from_node_bp(node, pos);
                init_lua(&lua_assets, &mut lnode);

                let mut node_list = vec![];

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

                if contains_audio(node) {
                    info!("contains audio");
                    node_list.push(AudioNode { connection: None });
                    ev_audio_change.send(AudioNodeChangeEvent(entity));
                }

                let mut ce = commands.entity(entity);

                node_list.iter().for_each(|item| {
                    ce.insert(item.clone());
                });

                info!("spawned node: {:?}", entity);
            }

            GenericNode::Native(node) => {
                panic!("not implemented")
            }
        }
    }
}

fn contains_audio(node: &LuaNode) -> bool {
    node.node
        .output_slots
        .iter()
        .find(|x| match x.signal_type {
            NodeType::SignalLink => true,
            _ => false,
        })
        .is_some()
}

fn construct_node_from_node_bp(node: &LuaNode, pos: Position) -> LuaNode {
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
