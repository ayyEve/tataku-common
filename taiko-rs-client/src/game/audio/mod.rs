use cpal::{
    Data, SampleRate,
    traits::{HostTrait, DeviceTrait, StreamTrait}
};

use symphonia::core::{
    audio::{AudioBufferRef, Signal},
    io::MediaSourceStream,
    probe::Hint
};

use std::{rc::Rc, sync::atomic::{AtomicUsize, Ordering}};
use std::sync::Arc;
use parking_lot::Mutex;

use itertools::Itertools;

const SOUND_LIST:[&'static str; 4] = [
    "audio/don.wav",
    "audio/kat.wav",
    "audio/bigdon.wav",
    "audio/bigkat.wav",
];

lazy_static::lazy_static!(
    static ref AUDIO: Arc<Mutex<Option<Audio>>> = Arc::new(Mutex::new(None));
);

pub struct Audio {
    
}
impl Audio {
    // todo: fix everything so nothing crashes and you can always change the device later etc
    pub fn setup() -> cpal::Stream {
        // temp
        let file = std::fs::File::open("audio/time-traveler.mp3")
        .expect("Failed to open file.");

        let source = MediaSourceStream::new(
            Box::new(file),
            Default::default()
        );

        let probe = symphonia::default::get_probe().format(
            Hint::new().with_extension("mp3"),
            source,
            &Default::default(),
            &Default::default(),
        )
        .expect("Failed to create probe");

        let mut reader = probe.format;

        let track = reader.default_track().unwrap();
        let track_id = track.id;

        println!("Sample Rate Track: {:?}", track.codec_params.sample_format);

        let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, &Default::default())
        .expect("Failed to get codecs and produce decoder.");

        let mut samples = Vec::new();
        let mut rate = 0;

        while let Ok(packet) = reader.next_packet() {
            if packet.track_id() != track_id { continue; }

            match decoder.decode(&packet) {
                Ok(AudioBufferRef::F32(f)) => {
                    if rate == 0 { rate = f.spec().rate };

                    let channel_0 = f.chan(0).into_iter().cloned();
                    let channel_1 = f.chan(1).into_iter().cloned();
                    samples.append(&mut channel_0.interleave(channel_1).collect_vec());
                }

                Ok(AudioBufferRef::S32(f)) => {
                    println!("wat, s32");
                }

                Err(e) => {
                    println!("sound oof: {:?}", e);
                }

                _ => {}
            }
        }

        let host = cpal::default_host();

        let device = host.default_output_device()
            .expect("No default output device available.");

        let mut supported_configs = device.supported_output_configs()
            .expect("Error while querying configs.");

        let supported_config_range = supported_configs.next()
            .expect("No supported config?");

        println!("Range Rate: {}-{}Hz", supported_config_range.min_sample_rate().0, supported_config_range.max_sample_rate().0);

        let supported_config = supported_config_range.with_max_sample_rate();

        let stream_rate = supported_config.sample_rate().0;

        println!("Sample Rate Stream: {}", stream_rate);

        let scale_factor = stream_rate as f32 / rate as f32;

        let mut scaled_samples = vec![0.0; (samples.len() as f32 * scale_factor + 1.0).ceil() as usize];

        // no, its all fucked idk
        for (old_index, sample) in samples.iter().enumerate() {
            let sd_len = samples.len();

            let new_index = old_index as f32 * scale_factor;
            let j = new_index.floor() as usize;
            let f = new_index.fract();

            let xm1 = samples[(old_index - 2) % sd_len];
            let x0  = samples[old_index       % sd_len];
            let x1  = samples[(old_index + 2) % sd_len];
            let x2  = samples[(old_index + 4) % sd_len];

            let c     = (x1 - xm1) * 0.5;
            let v     = x0 - x1;
            let w     = c + v;
            let a     = w + v + (x2 - x0) * 0.5;
            let b_neg = w + a;

            scaled_samples[j] = (((a * f) - b_neg) * f + c) * f + x0;
        }

        let count = AtomicUsize::new(0);

        const volume: f32 = 0.3;

        let stream = device.build_output_stream(
            &supported_config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // react to stream events and read or write stream data here.

                let index = count.load(Ordering::SeqCst);

                for (i, sample) in data.iter_mut().enumerate() {
                    *sample = volume * scaled_samples[(index + i) % scaled_samples.len()];
                }

                count.store((index + data.len()) % scaled_samples.len(), Ordering::SeqCst);
            },
            move |err| {
                println!("wat: {:?}", err);
            }
        )
        .expect("Failed to build output stream.");

        stream.play().unwrap();

        stream
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AudioState {
    Playing,
    Paused,
    Stopped
}
