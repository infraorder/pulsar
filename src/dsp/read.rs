use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

use atomic_float::AtomicF32;
use bevy::{asset::Assets, ecs::system::Res, log::info};
use knyst::{
    gen::Gen,
    prelude::{GenContext, GenState},
    Resources,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::components::lua::LuaAsset;

use super::{
    audio_graph::{Streamable, AUDIO_BUFFER, AUDIO_SIZE},
    AudioSendControl,
};

pub type ChanOut = Vec<Arc<AtomicF32>>;

pub struct Read {}

#[derive(Clone)]
pub struct StreamBuf {
    last_out: Vec<ChanOut>,
    out_idx: Arc<AtomicUsize>,
    lock: Arc<AtomicBool>,
}

pub struct ReadStream {
    should_swap: Arc<AtomicBool>,
    curr_chan: Arc<AtomicUsize>,

    stream_buf: [StreamBuf; 2],
}

impl ReadStream {
    fn generate_samples(&mut self, ctx: GenContext) {
        let buf = self.get_active_buf();

        let mut out_idx_offset = buf.out_idx.load(Ordering::Relaxed);

        buf.lock.swap(true, Ordering::Relaxed);

        buf.out_idx
            .swap(out_idx_offset + AUDIO_SIZE, Ordering::Relaxed);

        if buf.out_idx.load(Ordering::Relaxed) + ctx.block_size() > AUDIO_BUFFER {
            info!("maxed out buffer");

            buf.out_idx.swap(0, Ordering::Relaxed);
            out_idx_offset = 0;
        }

        (0..ctx.block_size()).into_iter().for_each(|i| {
            let in0 = ctx.inputs.read(0, i);
            let in1 = ctx.inputs.read(1, i);

            ctx.outputs.write(in0, 0, i);
            ctx.outputs.write(in1, 1, i);
            buf.last_out[0][out_idx_offset + i].swap(in0, Ordering::Relaxed);
            buf.last_out[1][out_idx_offset + i].swap(in1, Ordering::Relaxed);
        });

        if self.should_swap.load(Ordering::Relaxed) {
            self.should_swap.swap(false, Ordering::Relaxed);
            self.swap_chan();
            buf.lock.swap(false, Ordering::Relaxed);
        }
    }

    pub fn _get_inactive_buf(&self) -> &StreamBuf {
        let i = 1 - self.curr_chan.load(Ordering::Relaxed);

        &self.stream_buf[i]
    }

    pub fn get_active_buf(&self) -> &StreamBuf {
        &self.stream_buf[self.curr_chan.load(Ordering::Relaxed)]
    }

    pub fn swap_chan(&self) {
        let i = self.curr_chan.load(Ordering::Relaxed);

        self.curr_chan.swap(1 - i, Ordering::Relaxed);
    }
}

impl Gen for ReadStream {
    fn process(&mut self, ctx: GenContext, _resources: &mut Resources) -> GenState {
        self.generate_samples(ctx);

        GenState::Continue
    }

    fn num_inputs(&self) -> usize {
        2
    }

    fn num_outputs(&self) -> usize {
        2
    }
}

pub struct ReadControl {
    should_swap: Arc<AtomicBool>,
    curr_chan: Arc<AtomicUsize>,

    stream_buf: [StreamBuf; 2],
}

impl ReadControl {
    pub fn last_out(&self) -> Option<(usize, Vec<Vec<f32>>)> {
        let buf = self.get_inactive_buf();

        if buf.lock.load(Ordering::Relaxed) {
            panic!("HOW DID I GET HERE");
        }

        if buf.out_idx.load(Ordering::Relaxed) == 0 {
            return None;
        }

        let out_idx = buf.out_idx.load(Ordering::Relaxed);
        buf.out_idx.swap(0, Ordering::Relaxed);

        let out = Some((
            out_idx,
            buf.last_out
                .par_iter()
                .map(|o| {
                    o[0..out_idx]
                        .par_iter()
                        .map(|e| e.load(Ordering::Relaxed))
                        .collect()
                })
                .collect(),
        ));

        self.should_swap.swap(true, Ordering::Relaxed);

        out
    }

    pub fn get_inactive_buf(&self) -> &StreamBuf {
        let i = 1 - self.curr_chan.load(Ordering::Relaxed);

        &self.stream_buf[i]
    }
}

impl Streamable for Read {
    type Stream = ReadStream;
    type Control = ReadControl;

    fn to_stream(&mut self, _lua: &Res<Assets<LuaAsset>>) -> Option<AudioSendControl> {
        let curr_chan = Arc::new(AtomicUsize::new(0));
        let should_swap = Arc::new(AtomicBool::new(true));

        let stream_buf = [stream_buf(), stream_buf()];

        let control = ReadControl {
            stream_buf: stream_buf.clone(),
            curr_chan: curr_chan.clone(),
            should_swap: should_swap.clone(),
        };

        let stream = ReadStream {
            stream_buf,
            curr_chan,
            should_swap,
        };
        Some(AudioSendControl::Read((stream, control)))
    }
}

fn stream_buf() -> StreamBuf {
    StreamBuf {
        last_out: vec![
            [(); AUDIO_BUFFER]
                .map(|_| Arc::new(AtomicF32::new(0.0)))
                .to_vec(),
            [(); AUDIO_BUFFER]
                .map(|_| Arc::new(AtomicF32::new(0.0)))
                .to_vec(),
        ],
        out_idx: Arc::new(AtomicUsize::new(0)),
        lock: Arc::new(AtomicBool::new(false)),
    }
}
