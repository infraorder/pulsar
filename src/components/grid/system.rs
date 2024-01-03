use bevy::{
    asset::{AssetServer, Assets},
    ecs::{
        component::Component,
        system::{Commands, Query, Res, ResMut},
    },
    math::{Vec2, Vec3},
    prelude::{Deref, DerefMut},
    render::mesh::Mesh,
    sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
    time::{Time, Timer},
    transform::components::Transform,
};

use crate::InstancingBundle;

use super::GridBundle;

pub fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("sprites/sheet.png");
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::new(7., 9.),
        2,
        1,
        None,
        Some(Vec2::new(14., 0.)),
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let spundle = SpriteSheetBundle {
        texture_atlas: texture_atlas_handle,
        sprite: TextureAtlasSprite::new(1),
        transform: Transform::from_scale(Vec3::splat(6.0)),
        ..Default::default()
    };

    // Use only the subset of sprites in the sheet that make up the run animation

    commands.spawn(GridBundle {
        instancing: InstancingBundle {
            ..Default::default()
        },
        ..Default::default()
    });
}

#[derive(Component, Deref, DerefMut)]
pub struct GridAnimationTimer(Timer);

pub fn animate_grid(
    time: Res<Time>,
    mut query: Query<(&mut GridAnimationTimer, &mut TextureAtlasSprite)>,
) {
    for (mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = 1 - sprite.index;
        }
    }
}
