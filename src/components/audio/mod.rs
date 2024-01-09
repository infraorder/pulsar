pub mod system;

use bevy::ecs::system::Resource;

use crate::dsp::{ChainType, TChain};

#[derive(Resource, Clone)]
pub struct AudioGraph {
    pub chain: TChain,
}

impl AudioGraph {
    pub fn get_chain(&self) -> &Vec<TChain> {
        match self.chain.t.as_ref() {
            ChainType::ChainList(l) => l,
            _ => panic!("AudioGraph chain is not a ChainList"),
        }
    }

    pub fn get_chain_mut(&mut self) -> &mut Vec<TChain> {
        match self.chain.t.as_mut() {
            ChainType::ChainList(l) => l,
            _ => panic!("AudioGraph chain is not a ChainList"),
        }
    }
}
