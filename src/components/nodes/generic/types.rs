use bevy::ecs::{event::Event, entity::Entity};

#[derive(Event)]
pub struct AudioNodeChangeEvent {
    pub entity: Entity,
    pub slot_idx: usize,
}
