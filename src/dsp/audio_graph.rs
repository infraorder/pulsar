pub const AUDIO_BUFFER: usize = AUDIO_SIZE * 256;
pub const AUDIO_SIZE: usize = 64;

use bevy::{
    app::{PostUpdate, Update},
    asset::{AssetEvent, Assets},
    ecs::{component::TableStorage, event::EventReader, query::With, system::Res},
    prelude::{
        App, Commands, Component, Deref, DerefMut, Entity, NonSendMut, Plugin, Query, Without,
    },
};

use knyst::{
    audio_backend::{CpalBackend, CpalBackendOptions},
    controller::KnystCommands,
    gen::Gen,
    graph::{connection::InputBundle, Graph, GraphSettings, NodeId},
    inputs, knyst_commands,
    sphere::{KnystSphere, SphereSettings},
};

use crate::{
    components::lua::LuaAsset,
    dsp::{
        oscillators::Oscillator, read::Read, AudioControl as AC, AudioSend, AudioSendControl,
        Chain, Dsp,
    },
};

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<AudioOutput>()
            .add_systems(PostUpdate, play_audio)
            .add_systems(Update, update_audio);
    }
}

pub struct AudioOutput {
    // pub(crate) knyst: KnystCommands,
    _backend: CpalBackend,
    _error_receiver: std::sync::mpsc::Receiver<String>,
}

impl Default for AudioOutput {
    fn default() -> Self {
        let (error_sender, _error_receiver) = std::sync::mpsc::channel();

        let mut backend = CpalBackend::new(CpalBackendOptions::default())
            .unwrap_or_else(|err| panic!("Cannot initialize cpal backend. Error: {err}"));

        let _sphere = KnystSphere::start(
            &mut backend,
            SphereSettings {
                num_inputs: 0,
                num_outputs: 2,
                ..Default::default()
            },
            Box::new(move |error| {
                error_sender.send(format!("{error}")).unwrap();
            }),
        );

        Self {
            _error_receiver,
            _backend: backend,
        }
    }
}

impl AudioOutput {
    fn play_stream(&mut self, stream: Box<Vec<Box<AudioSend>>>) -> Vec<NodeId> {
        let mut chain_out: Vec<NodeId> = vec![];

        for stream in stream.into_iter() {
            match *stream {
                AudioSend::Read(stream) => chain_out.push(self.push(stream, chain_out.last())),
                AudioSend::Oscillator(stream) => {
                    chain_out.push(self.push(stream, chain_out.last()))
                }
                AudioSend::Output => todo!(),
            }
        }

        knyst_commands().connect(chain_out.last().unwrap().to_graph_out().channels(2));
        chain_out
    }

    fn push(&mut self, stream: impl Gen + Send + 'static, inputs: Option<&NodeId>) -> NodeId {
        let inputs = match inputs {
            None => inputs!(),
            Some(node_address) => {
                inputs!((0 ; node_address.out(0)), (1 ; node_address.out(1)))
            }
        };

        let id = knyst_commands().push(stream, inputs);

        id
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
pub struct AudioId(pub NodeId);

#[derive(Deref, DerefMut)]
pub struct AudioControl<T: Streamable>(T::Control);

impl<T: Streamable> Component for AudioControl<T> {
    type Storage = TableStorage;
}

pub trait Streamable: Send + Sync + 'static {
    type Stream: Gen + Send;
    type Control: Send + Sync;

    fn to_stream(
        &mut self,
        // k: &mut KnystCommands,
        lua: &Res<Assets<LuaAsset>>,
    ) -> Option<AudioSendControl>;
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
                Dsp::Input(i) => i.to_stream(&lua_assets),
                Dsp::Read(i) => i.to_stream(&lua_assets),
                Dsp::Output => Some(AudioSendControl::Output),
            })
            .map(|i| i.unwrap()) // TODO: handle None by failing entire chain
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

                    AudioSendControl::Output => {
                        res.0.push(Box::new(AudioSend::Output));
                        res.1.push(AC::Output);
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
                AC::Output => todo!(),
            });
    }
}

fn update_audio(
    mut commands: Commands,
    audio_query: Query<(Entity, &Audio, &AudioId), With<AudioControl<Oscillator>>>,
    mut lua_asset_event: EventReader<AssetEvent<LuaAsset>>,
    // audio_graph: NonSendMut<AudioOutput>,
) {
    for ev in lua_asset_event.read() {
        match ev {
            AssetEvent::LoadedWithDependencies { id: asset_id } => {
                for (entity, audio, audio_id) in audio_query.iter() {
                    audio
                        .chain
                        .items
                        .iter()
                        .for_each(|send_control| match send_control {
                            Dsp::Input(audio) => {
                                if audio
                                    .lua_handle
                                    .iter()
                                    .any(|handle| handle.id() == asset_id.to_owned())
                                {
                                    knyst_commands().free_node(audio_id.0.clone());
                                    commands.entity(entity).remove::<AudioId>();
                                    commands.entity(entity).remove::<AudioControl<Read>>();
                                    commands.entity(entity).remove::<AudioControl<Oscillator>>();
                                }
                            }
                            _ => (),
                        });
                }
            }
            _ => return,
        }
    }
}
