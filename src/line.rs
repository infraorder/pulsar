use bevy::ecs::component::Component;

use crate::audio_graph::AUDIO_SIZE;

pub const BUFFER_SIZE: usize = AUDIO_SIZE * 64;
pub const SPLIT_LEN: usize = AUDIO_SIZE * 16;

#[derive(Component, Debug, Clone)]
pub struct XYLine {
    pub buffer: [[f32; BUFFER_SIZE]; 2],
    pub index: usize,
}

impl Default for XYLine {
    fn default() -> Self {
        Self {
            buffer: [[(); BUFFER_SIZE].map(|_| 0.); 2],
            index: 0,
        }
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct SplitLine {
    pub buffer: [Vec<f32>; 2],
}
