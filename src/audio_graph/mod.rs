// #![warn(clippy::pedantic)]

pub mod asset_reader;

pub const AUDIO_BUFFER: usize = AUDIO_SIZE * 256;
pub const AUDIO_SIZE: usize = 64;

use bevy::{
    app::{PostUpdate, Update},
    asset::Assets,
    ecs::{
        component::TableStorage,
        system::{Res, ResMut},
    },
    prelude::{
        App, Commands, Component, Deref, DerefMut, Entity, NonSendMut, Plugin, Query, Without,
    }, log::info,
};

use knyst::{
    audio_backend::{CpalBackend, CpalBackendOptions},
    controller::{print_error_handler, KnystCommands},
    graph::{connection::InputBundle, Gen, NodeAddress},
    inputs,
    prelude::{AudioBackend, Graph, GraphSettings, ResourcesSettings},
    Resources,
};

use crate::dsp::{
    oscillators::Oscillator, read::Read, AudioControl as AC, AudioSend, AudioSendControl, Chain,
    Dsp,
};

use self::asset_reader::LuaAsset;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<AudioOutput>()
            .add_systems(PostUpdate, play_audio)
            .add_systems(Update, update_audio);
    }
}

pub struct AudioOutput {
    pub(crate) knyst: KnystCommands,
    _backend: CpalBackend,
}

impl Default for AudioOutput {
    fn default() -> Self {
        let mut backend = CpalBackend::new(CpalBackendOptions::default())
            .unwrap_or_else(|err| panic!("Cannot initialize cpal backend. Error: {err}"));

        let sample_rate = backend.sample_rate() as f32;
        let block_size = backend.block_size().unwrap_or(64);

        let resources = Resources::new(ResourcesSettings::default());

        info!("sample_rate = {}", backend.sample_rate());
        println!("num outputs - {}", backend.num_outputs());

        let graph = Graph::new(GraphSettings {
            // num_outputs: backend.num_outputs(),
            num_outputs: 2,
            block_size,
            sample_rate,
            ..Default::default()
        });

        let knyst = backend
            .start_processing(
                graph,
                resources,
                knyst::graph::RunGraphSettings::default(),
                Box::new(print_error_handler),
            )
            .unwrap_or_else(|err| panic!("Cannot start processing audio graph. Error: {err}"));

        Self {
            knyst,
            _backend: backend,
        }
    }
}

impl AudioOutput {
    fn play_stream(&mut self, stream: Box<Vec<Box<AudioSend>>>) -> Vec<NodeAddress> {
        let mut chain_out: Vec<NodeAddress> = vec![];

        for stream in stream.into_iter() {
            match *stream {
                AudioSend::Read(stream) => chain_out.push(self.push(stream, chain_out.last())),
                AudioSend::Oscillator(stream) => {
                    chain_out.push(self.push(stream, chain_out.last()))
                }
            }
        }

        self.knyst
            .connect(chain_out.last().unwrap().to_graph_out().channels(2));
        chain_out
    }

    fn push(
        &mut self,
        stream: impl Gen + Send + 'static,
        inputs: Option<&NodeAddress>,
    ) -> NodeAddress {
        let inputs = match inputs {
            None => inputs!(),
            Some(node_address) => {
                inputs!((0 ; node_address.out(0)), (1 ; node_address.out(1)))
            }
        };

        self.knyst.push(stream, inputs)
    }
}

pub struct Audio {
    chain: Chain,
}

impl Component for Audio {
    type Storage = TableStorage;
}

impl Audio {
    pub fn new(chain: Chain) -> Self {
        Self { chain }
    }
}

#[derive(Component)]
pub struct AudioId(pub NodeAddress);

#[derive(Deref, DerefMut)]
pub struct AudioControl<T: Streamable>(T::Control);

impl<T: Streamable> Component for AudioControl<T> {
    type Storage = TableStorage;
}

pub trait Streamable: Send + Sync + 'static {
    type Stream: Gen + Send;
    type Control: Send + Sync;

    fn to_stream(&mut self, k: &mut KnystCommands, lua: &Res<Assets<LuaAsset>>)
        -> Option<AudioSendControl>;
}

fn play_audio(
    mut commands: Commands,
    mut audio_query: Query<(Entity, &mut Audio), Without<AudioId>>,
    lua_assets: Res<Assets<LuaAsset>>,
    mut audio_graph: NonSendMut<AudioOutput>,
) {

    for (entity, mut audio) in audio_query.iter_mut() {
        let (stream, control) = audio
            .chain
            .items
            .iter_mut()
            .map(|i| match i {
                Dsp::Input(i) => i.to_stream(&mut audio_graph.knyst, &lua_assets),
                Dsp::Read(i) => i.to_stream(&mut audio_graph.knyst, &lua_assets),
            })
            .filter(|i| i.is_some())
            .map(|i| i.unwrap())
            .fold((vec![], vec![]), |mut res, i| {
                match i {
                    AudioSendControl::Read((stream, control)) => {
                        res.0.push(Box::new(AudioSend::Read(stream)));
                        res.1.push(AC::Read(control));
                    }

                    AudioSendControl::Oscillator((stream, control)) => {
                        res.0.push(Box::new(AudioSend::Oscillator(stream)));
                        res.1.push(AC::Oscillator(control));
                    }
                }

                res
            });

        if (stream.len() == 0) || (control.len() == 0) {
            continue;
        }

        let node_addresses = audio_graph.play_stream(Box::new(stream));

        control
            .into_iter()
            .zip(node_addresses.into_iter())
            .for_each(|(control, node_address)| match control {
                AC::Read(control) => {
                    commands
                        .entity(entity)
                        .insert((AudioId(node_address), AudioControl::<Read>(control)));
                }
                AC::Oscillator(control) => {
                    commands
                        .entity(entity)
                        .insert((AudioId(node_address), AudioControl::<Oscillator>(control)));
                }
            });
    }
}

fn update_audio(
    mut commands: Commands,
    audio_query: Query<(Entity, &Audio, &AudioId)>,
    lua_assets: ResMut<Assets<LuaAsset>>,
    mut audio_graph: NonSendMut<AudioOutput>,
) {
    for (entity, audio, audio_id) in audio_query.iter() {
        audio
            .chain
            .items
            .iter()
            .for_each(|send_control| match send_control {
                Dsp::Input(audio) => {
                    if lua_assets
                        .get(audio.lua_handle.clone())
                        .unwrap()
                        .script
                        .ne(&audio.lua_string)
                    {
                        audio_graph.knyst.free_node(audio_id.0.clone());
                        commands.entity(entity).remove::<AudioId>();
                    }
                }
                _ => (),
            });
    }
}
