use bevy::ecs::entity::Entity;

use self::{
    oscillators::{Oscillator, OscillatorControl, OscillatorStream},
    read::{Read, ReadControl, ReadStream},
};

pub mod audio_graph;
pub mod oscillators;
pub mod read;

#[derive(Clone)]
pub enum Dsp {
    Input(Oscillator),
    Read(Read),
}

pub enum AudioSendControl {
    Read((ReadStream, ReadControl)),
    Oscillator((OscillatorStream, OscillatorControl)),
}

pub enum AudioControl {
    Read(ReadControl),
    Oscillator(OscillatorControl),
}

pub enum AudioSend {
    Read(ReadStream),
    Oscillator(OscillatorStream),
}

pub struct Chain {
    pub items: Vec<Dsp>,
}

// TODO: switch to this
#[derive(Clone)]
pub enum ChainType {
    Dsp(Dsp),
    ChainList(Vec<TChain>),
    None,
}

#[derive(Clone)]
pub struct TChain {
    pub t: Box<ChainType>,
    pub e: Option<Entity>,
}

impl TChain {
    pub fn dsp(dsp: Dsp, e: Option<Entity>) -> Self {
        Self {
            t: Box::new(ChainType::Dsp(dsp)),
            e,
        }
    }

    pub fn vec(vec: Vec<TChain>, e: Option<Entity>) -> Self {
        Self {
            t: Box::new(ChainType::ChainList(vec)),
            e,
        }
    }
}
