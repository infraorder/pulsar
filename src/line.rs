use bevy::ecs::component::Component;

use crate::audio_graph::AUDIO_SIZE;

pub const BUFFER_SIZE: usize = AUDIO_SIZE * 64;

#[derive(Component, Debug, Clone)]
pub struct AudioLine {
    pub line: [[f32; BUFFER_SIZE]; 2],
    pub index: usize,
}

impl Default for AudioLine {
    fn default() -> Self {
        Self {
            line: [[(); BUFFER_SIZE].map(|_| 0.); 2],
            index: 0,
        }
    }
}
