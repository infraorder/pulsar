use bevy::{
    asset::AssetServer,
    ecs::{
        bundle::Bundle,
        component::Component,
        system::{Commands, Res},
    },
    hierarchy::BuildChildren,
    math::{Vec2, Vec3},
    sprite::{Sprite, SpriteBundle},
    text::{Text, Text2dBounds, Text2dBundle, TextAlignment, TextSection, TextStyle},
    transform::components::Transform,
};

use super::types::Display;

pub fn spawn_text2d<T: Display + Component>(asset_server: &Res<AssetServer>, p: &T) -> impl Bundle {
    let box_size = Vec2::new(20.0, 30.0);

    let name = p.name().clone();
    let color = p.get_inert();

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
        transform: Transform::from_translation(Vec3::Z),
        ..Default::default()
    }
}

pub fn spawn_e_with_c<T: Display + Component>(
    cmd: &mut Commands,
    asset_server: &Res<AssetServer>,
    p: T,
    c: &mut Vec<T>,
) {
    let mut ce = cmd.spawn(spawn_collider_e(p));

    ce.with_children(|builder| {
        builder.spawn(spawn_text2d(asset_server, &p));
    });

    ce.with_children(|builder| {
        (0..c.len()).for_each(|x| {
            let t = c.pop().unwrap();
            builder.spawn(spawn_collider_e(t));
        });
    });
}

pub fn spawn_collider_e<T: Display + Component>(p: T) -> impl Bundle {
    let box_size = Vec2::new(20.0, 30.0);
    let pos = p.pos().to_vec2() * -box_size;
    let bcol = p.get_inert().foreground.0;

    (
        p,
        SpriteBundle {
            sprite: Sprite {
                color: bcol,
                custom_size: Some(box_size - Vec2::new(5.0, 5.0)),
                ..Default::default()
            },
            transform: Transform::from_translation(pos.extend(0.0)),
            ..Default::default()
        },
    )
}
