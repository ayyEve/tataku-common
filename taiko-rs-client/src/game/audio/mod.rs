use cpal::{
    Data, SampleRate,
    traits::{HostTrait, DeviceTrait, StreamTrait}
};

use rubato::{SincFixedIn, Resampler, InterpolationParameters, InterpolationType, WindowFunction};
use symphonia::core::{
    audio::{AudioBufferRef, Signal},
    io::MediaSourceStream,
    probe::Hint
};

use std::{rc::Rc, sync::atomic::{AtomicUsize, Ordering}, time::Instant};
use std::sync::Arc;
use parking_lot::Mutex;

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

        // temp
        let mult = 0.6;
        let file = "songs/288610 Nightstep - Circles/Nightstep - Circles.mp3";
        let file = std::fs::File::open(file)
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

        println!("Sample Rate Track: {:?}", track.codec_params.sample_rate);

        let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, &Default::default())
            .expect("Failed to get codecs and produce decoder.");

        let mut samples = Vec::new();
        let mut track_rate = 0;
        let mut channels = 0;

        while let Ok(packet) = reader.next_packet() {
            if packet.track_id() != track_id { continue; }

            match decoder.decode(&packet) {
                Ok(AudioBufferRef::F32(f)) => {
                    if track_rate == 0 { track_rate = f.spec().rate; }
                    if channels == 0 { 
                        channels = f.spec().channels.count();

                        // Populate channels vec
                        for _ in 0..channels {
                            samples.push(Vec::new());
                        }
                    }

                    for chan in 0..channels {
                        samples[chan].append(&mut f.chan(chan).to_vec());
                    }
                }

                Ok(AudioBufferRef::S32(_f)) => {
                    println!("wat, s32");
                }

                Err(e) => {
                    println!("sound oof: {:?}", e);
                }

                _ => {}
            }
        }

        // todo: Potentially mess with these, shove them in a lazy_static?
        let params = InterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            oversampling_factor: 128,
            interpolation: InterpolationType::Linear,
            window: WindowFunction::Blackman
        };


        let sample_len = samples[0].len();
        let resampled_audio = if stream_rate != track_rate {
            let mut resampler = SincFixedIn::new(
                stream_rate as f64 / (track_rate as f64 * mult),
                params,
                sample_len,
                channels
            );
            resampler.process(samples.as_slice())
                .expect("Failed to resample audio.")
        } else {
            samples
        };

        let mut interleaved_samples = Vec::new();

        for i in 0..resampled_audio[0].len() {
            for chan in 0..channels {
                interleaved_samples.push(resampled_audio[chan].get(i).map(|&x| x).unwrap_or(0.0));
            }
        }
        
        let count = AtomicUsize::new(0);
        const volume: f32 = 0.5;

        let stream = device.build_output_stream(
            &supported_config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // react to stream events and read or write stream data here.

                let index = count.load(Ordering::SeqCst);

                for (i, sample) in data.iter_mut().enumerate() {
                    *sample = volume * interleaved_samples[(index + i) % interleaved_samples.len()];
                }

                count.store((index + data.len()) % interleaved_samples.len(), Ordering::SeqCst);
            },
            move |err| {
                println!("wat: {:?}", err);
            }
        )
        .expect("Failed to build output stream.");
        
        println!("build output stream done");

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
