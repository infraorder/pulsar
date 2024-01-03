use core::panic;
use std::{
    f32::consts::TAU,
    sync::{atomic::Ordering, Arc},
    time::{Duration, Instant},
};

use atomic_float::AtomicF32;
use bevy::{
    asset::{Assets, Handle},
    ecs::system::Res,
    log::{info, trace},
};
use knyst::{
    gen::Gen,
    prelude::{GenContext, GenState},
    Resources,
};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use rlua::{Function, Lua, Result};

use crate::{
    components::lua::LuaAsset,
    lua::{init_instance, load_fn},
};

use super::{
    audio_graph::{Streamable, AUDIO_SIZE},
    AudioSendControl,
};

const OUT: &str = "OUT_FN";

pub struct Oscillator {
    pub lua_handle: Handle<LuaAsset>,
    pub lua_util_handle: Handle<LuaAsset>,
    pub lua_string: String,
    pub frequency_hz: f32,
}

pub struct OscillatorStream {
    frequency: Arc<AtomicF32>,
    luas: [Box<Lua>; AUDIO_SIZE],

    // duration stuff
    duration: [Duration; AUDIO_SIZE],
    duration_idx: usize,
}

impl OscillatorStream {
    fn generate_samples(&mut self, sample_rate: f32, ctx: GenContext) {
        let interval = 1.0 / sample_rate;

        let frequency = self.frequency.load(Ordering::Relaxed);

        let t_size = interval * (ctx.block_size() as f32);

        (0..ctx.block_size())
            .into_par_iter()
            .zip(&mut self.luas)
            .map(|(i, lua)| {
                let t = interval * i as f32;
                let default: (f32, f32) = (0.0, 0.0);

                match call_lua(&lua, t, frequency, t_size) {
                    Ok(out) => (i, out),
                    Err(_) => (i, default),
                }
            })
            .collect::<Vec<(usize, (f32, f32))>>()
            .into_iter()
            .for_each(|(i, (out0, out1))| {
                let out0 = norm(out0);
                let out1 = norm(out1);

                ctx.outputs.write(out0, 0, i);
                ctx.outputs.write(out1, 1, i);
            });
    }
}

impl Gen for OscillatorStream {
    fn process(&mut self, ctx: GenContext, _resources: &mut Resources) -> GenState {
        let start = Instant::now();

        let sample_rate = ctx.sample_rate;
        self.generate_samples(sample_rate, ctx);

        let duration = start.elapsed();
        self.duration[self.duration_idx] = duration;

        trace!("Time elapsed in expensive_function() is: {:?}", duration);
        trace!(
            "Average time elapsed: {:?}",
            self.duration.iter().sum::<Duration>() / AUDIO_SIZE as u32
        );

        self.duration_idx += 1;

        if self.duration_idx == AUDIO_SIZE {
            self.duration_idx = 0;
        }

        GenState::Continue
    }

    fn num_inputs(&self) -> usize {
        0
    }

    fn num_outputs(&self) -> usize {
        2
    }
}

pub struct OscillatorControl {
    frequency: Arc<AtomicF32>,
    time: Arc<AtomicF32>,
}

impl OscillatorControl {
    pub fn frequency(&self) -> f32 {
        self.frequency.load(Ordering::Relaxed) / TAU
    }

    pub fn set_frequency(&self, frequency_hz: f32) {
        self.frequency.store(frequency_hz * TAU, Ordering::Relaxed);
    }

    pub fn set_time(&self, time: f32) {
        self.time.store(time * TAU, Ordering::Relaxed);
    }
}

impl Streamable for Oscillator {
    type Stream = OscillatorStream;
    type Control = OscillatorControl;

    fn to_stream(
        &mut self,
        // _knyst: &mut KnystCommands,
        lua_assets: &Res<Assets<LuaAsset>>,
    ) -> Option<AudioSendControl> {
        let custom_asset = lua_assets.get(self.lua_handle.clone());

        if custom_asset.is_none() {
            return None;
        }

        let custom_asset = custom_asset.unwrap();

        info!("Custom asset loaded: {:?}", custom_asset.script);

        self.lua_string = custom_asset.script.to_owned();

        let utils = lua_assets.get(self.lua_util_handle.clone());

        if utils.is_none() {
            return None;
        }

        let hood = utils.unwrap();

        info!("Custom asset loaded: {:?}", hood.script);

        let luas = [(); AUDIO_SIZE].map(|_| {
            let lua = init_instance();
            load_fn(&lua, "lua_pulse", &hood.script);
            load_fn(&lua, "lua_pulse", &custom_asset.script);

            Box::new(lua)
        });

        let frequency = Arc::new(AtomicF32::new(self.frequency_hz * TAU));

        let time = Arc::new(AtomicF32::new(0.0)); // will be received from bevy instance;

        let control = OscillatorControl {
            frequency: frequency.clone(),

            time: time.clone(),
        };

        let stream = OscillatorStream {
            frequency,

            luas,

            duration: [(); AUDIO_SIZE].map(|_| Duration::from_secs(0)),
            duration_idx: 0,
        };
        Some(AudioSendControl::Oscillator((stream, control)))
    }
}

fn call_lua(lua: &Box<Lua>, i: f32, frequency: f32, t: f32) -> Result<(f32, f32)> {
    lua.context(|ctx| {
        let function: Function = ctx.globals().get(OUT)?;

        let out = function.call::<_, (f32, f32)>((i, frequency, t));

        if out.is_err() {
            info!("OUT ERROR -> {:?}", out);
        }

        Ok(out?)
    })
}

fn norm(input: f32) -> f32 {
    match input {
        x if x <= 1. && x >= -1. => x,
        x if x > 1. => 1.,
        x if x < -1. => -1.,
        _ => panic!("should not get here"),
    }
}
