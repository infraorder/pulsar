use std::ops::Mul;

use bevy::{
    asset::AssetServer,
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        system::{Commands, Res},
    },
    hierarchy::{BuildChildren, ChildBuilder},
    math::{Vec2, Vec3},
    render::{color::Color, view::RenderLayers},
    sprite::{Sprite, SpriteBundle},
    text::{Text, Text2dBounds, Text2dBundle, TextAlignment, TextSection, TextStyle},
    transform::components::Transform,
};

use crate::{components::config::ConfigAsset, UI_TARGET};

use super::types::{
    ColorPair, Display, InputSlot, LuaNode, OutputSlot, PColor, SlotNode, SlotType,
};

pub fn spawn_text2d(
    config: &Res<ConfigAsset>,
    asset_server: &Res<AssetServer>,
    name: String,
    color: ColorPair,
) -> impl Bundle {
    let box_size = Vec2::new(20.0, 30.0);

    Text2dBundle {
        text: Text {
            sections: vec![TextSection::new(
                name,
                TextStyle {
                    font: asset_server.load("fonts/pulsar_font.ttf"),
                    font_size: 20.0,
                    color: color.foreground.0,
                },
            )],
            alignment: TextAlignment::Center,
            ..Default::default()
        },
        text_2d_bounds: Text2dBounds {
            // Wrap text in the rectangle
            size: box_size,
        },
        // ensure the text is drawn on top of the box
        transform: Transform::from_translation(Vec3::new(1.1, -10.0, 5.0)),
        ..Default::default()
    }
}

pub fn spawn_node_with_children<
    T: Display + Component,
    S: Display + Component,
    V: Display + Component,
>(
    config: &Res<ConfigAsset>,
    cmd: &mut Commands,
    asset_server: &Res<AssetServer>,
    node: T,
    input_slots: &mut Vec<S>,
    output_slots: &mut Vec<V>,
) {
    let e = spawn_node_with_text(config, cmd, asset_server, node);
    let mut ce = cmd.entity(e);

    ce.with_children(|builder| {
        (0..input_slots.len()).for_each(|_| {
            let t = input_slots.pop().unwrap();
            spawn_child_node_with_text(config, builder, asset_server, t);
        });
    });

    ce.with_children(|builder| {
        (0..output_slots.len()).for_each(|_| {
            let t = output_slots.pop().unwrap();
            spawn_child_node_with_text(config, builder, asset_server, t);
        });
    });
}

pub fn spawn_node_with_text<T: Display + Component>(
    config: &Res<ConfigAsset>,
    cmd: &mut Commands,
    asset_server: &Res<AssetServer>,
    node: T,
) -> Entity {
    let node_name = node.name().clone();
    let node_color = node.get_inert();

    let mut ce = cmd.spawn((spawn_node(config, node), RenderLayers::layer(UI_TARGET)));

    ce.with_children(|builder| {
        builder.spawn(spawn_text2d(
            config,
            asset_server,
            node_name.to_string(),
            node_color,
        ));
    });

    return ce.id();
}

pub fn spawn_child_node_with_text<T: Display + Component>(
    config: &Res<ConfigAsset>,
    cmd: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    node: T,
) -> Entity {
    let node_name = node.name().clone();
    let node_color = node.get_inert();

    let mut ce = cmd.spawn((spawn_node(config, node), RenderLayers::layer(UI_TARGET)));

    ce.with_children(|builder| {
        builder.spawn(spawn_text2d(
            config,
            asset_server,
            node_name.to_string(),
            node_color,
        ));
    });

    return ce.id();
}

pub fn spawn_node<T: Display + Component>(config: &Res<ConfigAsset>, node: T) -> impl Bundle {
    let box_size = Vec2::new(config.grid_offset_x, config.grid_offset_y);
    let pos = node.pos().to_vec2() * -box_size;
    let bcol = node.get_inert().background.0;

    (
        node,
        SpriteBundle {
            sprite: Sprite {
                color: bcol,
                custom_size: Some(box_size - Vec2::new(5.0, 5.0)),
                ..Default::default()
            },
            transform: Transform::from_translation(pos.extend(0.0).mul(-1.)),
            ..Default::default()
        },
    )
}

pub fn create_default_components(c: LuaNode) -> (LuaNode, Vec<InputSlot>, Vec<OutputSlot>) {
    let mut t = Vec::new();
    let mut s = Vec::new();

    c.node.output_slots.iter().enumerate().for_each(|(i, os)| {
        t.push(OutputSlot {
            idx: i,
            slot: SlotNode {
                slot_type: os.slot_type.clone(),
                signal_type: os.signal_type.clone(),
                display: get_slot_name(true, &os.slot_type),
                pos: os.pos.clone(),
                active: slot_active(),
                inert: slot_inert(),
                inactive: slot_inactive(),
            },
        });
    });

    c.node.slots.iter().enumerate().for_each(|(i, os)| {
        s.push(InputSlot {
            idx: i,
            slot: SlotNode {
                slot_type: os.slot_type.clone(),
                signal_type: os.signal_type.clone(),
                display: get_slot_name(false, &os.slot_type),
                pos: os.pos.clone(),
                active: slot_active(),
                inert: slot_inert(),
                inactive: slot_inactive(),
            },
        });
    });

    return (c, s, t);
}

pub fn get_slot_name(is_output: bool, slot_type: &SlotType) -> String {
    match slot_type {
        SlotType::F32 => format!("{}F", if is_output { "O" } else { "I" }),
        SlotType::I32 => format!("{}I", if is_output { "O" } else { "I" }),
        SlotType::F32x2 => format!("{}G", if is_output { "O" } else { "I" }),
        SlotType::I32x2 => format!("{}J", if is_output { "O" } else { "I" }),
        SlotType::Bang => format!("{}!", if is_output { "O" } else { "I" }),
        SlotType::None => format!("{}_", if is_output { "O" } else { "I" }),
    }
}

// TODO: give these proper colors
pub fn slot_active() -> ColorPair {
    ColorPair {
        foreground: PColor(Color::rgb(1.0, 0.0, 0.0)),
        background: PColor(Color::rgb(0.0, 1.0, 0.0)),
    }
}

pub fn slot_inactive() -> ColorPair {
    ColorPair {
        foreground: PColor(Color::rgb(0.0, 0.0, 1.0)),
        background: PColor(Color::rgb(1.0, 1.0, 0.0)),
    }
}

pub fn slot_inert() -> ColorPair {
    ColorPair {
        foreground: PColor(Color::rgb(1.0, 1.0, 0.0)),
        background: PColor(Color::rgb(0.0, 0.0, 1.0)),
    }
}
