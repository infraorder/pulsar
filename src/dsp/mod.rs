use bevy::ecs::entity::Entity;
use knyst::graph::NodeId;

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
    Output,
}

pub enum AudioSendControl {
    Read((ReadStream, ReadControl)),
    Oscillator((OscillatorStream, OscillatorControl)),
    Output,
}

pub enum AudioControl {
    Read(ReadControl),
    Oscillator(OscillatorControl),
    Output,
}

pub enum AudioSend {
    Read(ReadStream),
    Oscillator(OscillatorStream),
    Output,
}

pub struct Chain {
    pub items: Vec<Dsp>,
}
