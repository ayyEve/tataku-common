use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};

use std::sync::{Arc, Weak};

use std::time::Instant;

use std::collections::HashMap;
use std::path::Path;

mod sound; use sound::*;
mod handle; pub use handle::*;
mod queue; use queue::*;
mod instance; use instance::*;
mod utils;

use crate::Settings;

const SOUND_LIST:[&'static str; 4] = [
    "audio/don.wav",
    "audio/kat.wav",
    "audio/bigdon.wav",
    "audio/bigkat.wav",
];

lazy_static::lazy_static!(
    static ref AUDIO: Arc<Audio> = Arc::new(Audio::setup());

    static ref PRELOADED_SOUNDS: HashMap<String, Sound> = {
        let mut sounds:HashMap<String, Sound> = HashMap::new();

        for sound in SOUND_LIST.iter() {
            let sound_name = Path::new(sound).file_stem().unwrap().to_str().unwrap();
            println!("loading audio file {}", sound_name);

            let sound = Sound::load(sound);

            sounds.insert(sound_name.to_owned(), sound);
        }

        sounds
    };

);

pub struct Audio {
    queue: Arc<AudioQueueController>,

    sample_rate: u32,
}
impl Audio {
    // todo: fix everything so nothing crashes and you can always change the device later etc
    pub fn setup() -> Self {
        let host = cpal::default_host();

        let device = host.default_output_device()
            .expect("No default output device available.");

        let mut supported_configs = device.supported_output_configs()
            .expect("Error while querying configs.");

        let supported_config_range = supported_configs.next()
            .expect("No supported config?");

        println!("Range Rate: {}-{}Hz", supported_config_range.min_sample_rate().0, supported_config_range.max_sample_rate().0);

        let supported_config = supported_config_range.with_max_sample_rate();

        let sample_rate = supported_config.sample_rate().0;

        println!("Sample Rate Stream: {}", sample_rate);

        let (controller, mut queue) = AudioQueue::new();

        std::thread::spawn(move || {   
            let stream = device.build_output_stream(
                &supported_config.into(),
                move |data: &mut [f32], info: &cpal::OutputCallbackInfo| {
                    // react to stream events and read or write stream data here.
                    let instant = Instant::now();

                    let timestamp = info.timestamp();

                    let delay = match timestamp.playback.duration_since(&timestamp.callback) {
                        Some(d) => d.as_secs_f32() * 1000.0,
                        None => {
                            println!("uh oh, none delay");
                            0.0
                        }
                    };

                    for sample in data.iter_mut() {
                        *sample = queue.next().unwrap_or(0.0);
                    }

                    queue.set_delay(delay + instant.elapsed().as_secs_f32() * 1000.0);
                },
                move |err| {
                    println!("wat: {:?}", err);
                }
            )
            .expect("Failed to build output stream.");

            stream.play().unwrap();

            std::thread::park();
        });       

        Self {
            queue: controller,
            sample_rate
        }
    }

    fn play_sound(sound: Sound) -> Weak<AudioHandle> {
        let instance = AudioInstance::new(sound, AUDIO.sample_rate, 1.0);
        
        let handle = Arc::downgrade(&instance.handle);

        AUDIO.queue.add(instance);

        handle
    }

    pub fn play(path: impl AsRef<str>) -> Weak<AudioHandle> {
        Audio::play_sound(Sound::load(path.as_ref()))
    }

    pub fn play_raw(bytes: Vec<u8>) -> Weak<AudioHandle> {
        Audio::play_sound(Sound::load_raw(bytes))
    }

    pub fn play_preloaded(name: impl AsRef<str>) -> Option<Weak<AudioHandle>> {
        PRELOADED_SOUNDS.get(name.as_ref()).map(|x| {
            let handle = Audio::play_sound(x.clone());

            let upgraded = handle.upgrade().unwrap();
            upgraded.set_volume(Settings::get().get_effect_vol());
            upgraded.play();
            handle
        })
    }
}