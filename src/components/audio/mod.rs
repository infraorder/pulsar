pub mod system;

use bevy::ecs::system::Resource;

use crate::dsp::TChain;

#[derive(Resource, Clone, Default)]
pub struct AudioGraph {
    pub chain: TChain,
}
