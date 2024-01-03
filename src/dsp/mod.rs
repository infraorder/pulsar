use self::{
    oscillators::{Oscillator, OscillatorControl, OscillatorStream},
    read::{Read, ReadControl, ReadStream},
};

pub mod audio_graph;
pub mod oscillators;
pub mod read;

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
