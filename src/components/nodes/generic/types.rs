use bevy::ecs::{event::Event, entity::Entity};

#[derive(Event)]
pub struct AudioNodePulseEvent {
    pub entity: Entity,
    pub slot_idx: usize,
}
