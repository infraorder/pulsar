use bevy::{
    ecs::{bundle::Bundle, component::Component},
    sprite::SpriteBundle,
};

#[derive(Component, Default, Clone)]
pub struct Player;

#[derive(Bundle, Default, Clone)]
pub struct PlayerBundle {
    pub player: Player,
    pub sprite: SpriteBundle,
}
