use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::{io::{Cursor, Read}, fs::File, path::Path};

use rodio::{Decoder, OutputStream, Sink, OutputStreamHandle, Source};
use crate::game::Settings;

use parking_lot::RwLock;

const SOUND_LIST:[&'static str; 4] = [
    "audio/don.wav",
    "audio/kat.wav",
    "audio/bigdon.wav",
    "audio/bigkat.wav"
];

lazy_static::lazy_static! {
    static ref SOUNDS: HashMap<String, AudioData> = {
        let mut sounds:HashMap<String, AudioData> = HashMap::new();

        for sound in SOUND_LIST.iter() {
            let sound_name = Path::new(sound).file_stem().unwrap().to_str().unwrap();
            println!("loading audio file {}", sound_name);

            // Read raw binary data from the file 
            let mut data = Vec::new();
            File::open(sound).unwrap().read_to_end(&mut data).unwrap();
            let cur = Cursor::new(data);

            // Decode and convert to appropriate audio samples which can be used again later
            let source = Decoder::new(cur).unwrap();
            let converted = source.convert_samples();
            
            let channels = converted.channels();
            let sample_rate = converted.sample_rate();
            let duration = converted.total_duration();

            let audio = AudioData {
                samples: Arc::new(converted.collect()),
                channels,
                sample_rate,
                duration
            };

            sounds.insert(sound_name.to_owned(), audio);
        }

        sounds
    };
    static ref STREAM: Arc<Mutex<Option<OutputStreamHandle>>> = Arc::new(Mutex::new(None));
}

#[derive(Clone)]
pub struct AudioData {
    pub samples: Arc<Vec<f32>>,
    pub channels: u16,
    pub sample_rate: u32,
    pub duration: Option<Duration>,
}

#[derive(Clone)]
pub struct AudioInstance(Arc<RwLock<AudioInstanceInner>>);

struct AudioInstanceInner {
    data: AudioData,
    current_index: usize,

    first: bool,
    timer: Instant,
    delay: f64,

    state: AudioState,
}

impl AudioInstance {
    pub fn new(data: AudioData) -> Self {
        Self (Arc::new(RwLock::new(AudioInstanceInner {
            data,
            current_index: 0,

            first: true,
            timer: Instant::now(),
            delay: 0.0,

            state: AudioState::Stopped,
        })))
    }

    pub fn set_position(&self, duration: Duration) {
        let mut lock = self.0.write();
        println!("previous index: {}", lock.current_index);
        lock.current_index = (duration.as_secs_f64() * lock.data.sample_rate as f64 * lock.data.channels as f64).floor() as usize;
        println!("new index: {}", lock.current_index);
    }

    pub fn current_position(&self) -> Duration {
        let lock = self.0.read();
        Duration::from_secs_f64(lock.current_index as f64 / (lock.data.sample_rate as f64 * lock.data.channels as f64))
    }

    pub fn state(&self) -> AudioState {
        let lock = self.0.read();
        lock.state.clone()
    }

    pub fn delay(&self) -> f64 {
        self.0.read().delay
    }

    pub fn pause(&self) {
        let mut lock = self.0.write();
        lock.state = AudioState::Paused;
        lock.first = false;
        lock.delay = 0.0;
    }

    pub fn play(&self) {
        let mut lock = self.0.write();
        lock.state = AudioState::Playing;
        lock.first = true;
        lock.timer = Instant::now();
    }

    pub fn stop(&self) {
        self.pause();
        let mut lock = self.0.write();
        lock.current_index = 0;
        lock.state = AudioState::Stopped;
    }
}

impl Iterator for AudioInstance {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let mut lock = self.0.write();
        match lock.state {
            AudioState::Paused => Some(0.0),
            AudioState::Stopped => None,
            AudioState::Playing => {
                if lock.current_index < lock.data.samples.as_ref().len() {
                    let item = lock.data.samples.as_ref()[lock.current_index];
        
                    lock.current_index += 1;
        
                    if lock.first {
                        lock.delay = lock.timer.elapsed().as_secs_f64() * 1000.0;
                        lock.first = false;
                        println!("first, delay: {}ms", lock.delay);
                        println!("first, current index: {}", lock.current_index);
                    }

                    Some(item)
                }
                else {
                    None
                }
            }
        }
    }
}

impl Source for AudioInstance {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        let lock = self.0.read();
        lock.data.channels
    }

    fn sample_rate(&self) -> u32 {
        let lock = self.0.read();
        lock.data.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        let lock = self.0.read();
        lock.data.duration
    }
}

pub struct Audio;
impl Audio {
    /// play a sound effect. uses settings.get_effect_vol() for volume
    pub fn play(name:&str) {
        if let Some((audio_instance, sink)) = Audio::load_sink(name) {
            audio_instance.play();
            sink.set_volume(Settings::get().get_effect_vol());
            sink.play();
            sink.detach();
        }
    }

    pub fn load_sink(name:&str) -> Option<(AudioInstance, Sink)> {

        // check if we don't have the audio pre-loaded
        if !SOUNDS.contains_key(name) {
            if !Path::new(name).exists() {
                println!("[Audio] File not found! {}. No audio will be played", name);
                return None;
            }

            // load from file manually
            let mut data = Vec::new();
            File::open(name).unwrap().read_to_end(&mut data).unwrap();
            let cur = Cursor::new(data);

            let lock = STREAM.lock().unwrap();
            let stream_handle = lock.as_ref().unwrap();

            // Decode that sound file into a source
            let source = Decoder::new(cur).unwrap();

            let converted = source.convert_samples();
            
            let channels = converted.channels();
            let sample_rate = converted.sample_rate();
            let duration = converted.total_duration();

            let audio = AudioData {
                samples: Arc::new(converted.collect()),
                channels,
                sample_rate,
                duration
            };

            let source = AudioInstance::new(audio);

            let sink = Sink::try_new(&stream_handle).unwrap();
            sink.append(source.clone());
            sink.pause();
            return Some((source, sink));
        }

        // otherwise load from cache
        let audio = SOUNDS.get(name).unwrap().clone();
        let lock = STREAM.lock().unwrap();
        let stream_handle = lock.as_ref().unwrap();

        // Decode that sound file into a source
        let source = AudioInstance::new(audio);
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.append(source.clone());
        sink.pause();
        Some((source, sink))
    }

    pub fn from_raw(data:Vec<u8>) -> (AudioInstance, Sink) {
    
        let cur = Cursor::new(data);

        let lock = STREAM.lock().unwrap();
        let stream_handle = lock.as_ref().unwrap();

        // Decode that sound file into a source
        let source = Decoder::new(cur).unwrap();

        let converted = source.convert_samples();
            
        let channels = converted.channels();
        let sample_rate = converted.sample_rate();
        let duration = converted.total_duration();

        let audio = AudioData {
            samples: Arc::new(converted.collect()),
            channels,
            sample_rate,
            duration
        };

        let source = AudioInstance::new(audio);

        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.append(source.clone());
        sink.pause();
        (source, sink)
    }

    pub fn setup() -> OutputStream {
        let (stream, handle) = OutputStream::try_default().unwrap();
        STREAM.lock().unwrap().replace(handle);
        stream
    }
}


/// "clonable" wrapper for sound effects
pub struct SoundEffect {
    /// name of the sound effect
    name: String,
    /// player for the sound effect
    pub sink: Option<Sink>,
    /// The audio instance controller for the sound effect
    pub audio_instance: Option<AudioInstance>,
    // when was the effect started?

    // effect volume
    volume:f32
}
impl SoundEffect {
    pub fn new(sound:&str) -> SoundEffect {
        SoundEffect {
            name: sound.to_owned(),
            // none as it is loaded on play (since state is stopped)
            sink: None, //Audio::load_sink(sound), 
            audio_instance: None, 
            volume: 1.0
        }
    }
    pub fn new_empty() -> SoundEffect {
        Self::new("")
    }

    pub fn state(&self) -> AudioState {
        match &self.audio_instance {
            Some(audio_instance) => audio_instance.state().clone(),
            None => AudioState::Stopped
        }
    }

    pub fn duration(&self) -> u64 {
        match &self.audio_instance {
            Some(audio_instance) => {
                match audio_instance.state() {
                    AudioState::Stopped => 0,
                    _ => audio_instance.current_position().as_millis() as u64
                }
            }
            None => 0,
        }
    }

    pub fn set_volume(&mut self, vol:f32) {
        self.volume = vol;
        match self.state() {
            AudioState::Stopped => {},
            _ => {
                self.sink.as_ref().unwrap().set_volume(vol);
            }
        }
    }

    pub fn play(&mut self) {
        match self.state() {
            AudioState::Playing => {}, // do nothing, its already playing
            AudioState::Paused => { // resume the sink
                println!("pausweedd");
                self.audio_instance.as_ref().unwrap().play();
                self.sink.as_ref().unwrap().play();
            },
            AudioState::Stopped => { // reload the sink and play it
                if let Some((new_audio_instance, sink)) = Audio::load_sink(&self.name) {
                    println!("stoppppped");
                    new_audio_instance.play();
                    sink.set_volume(self.volume);
                    sink.play();
                    self.sink = Some(sink);
                    self.audio_instance = Some(new_audio_instance)
                }
            },
        }
        
        
    }
    pub fn pause(&mut self) {
        match self.state() {
            AudioState::Playing => {
                self.sink.as_ref().unwrap().pause();
                self.audio_instance.as_ref().unwrap().play();
            },
            _ => {}
        }
    }
    pub fn stop(&mut self) {
        match self.state() {
            AudioState::Stopped => {}, // do nothing, its already stopped
            _ => {
                self.sink.as_ref().unwrap().stop();
                self.audio_instance.as_ref().unwrap().stop();
            },
        }
    }
    
    pub fn _restart(&mut self) {
        self.stop();
        self.play();
    }
}
impl Clone for SoundEffect {
    /// create a clone of this audio device
    /// note that its more of a duplication, as all values are reset to default
    fn clone(&self) -> Self {
        let name = &self.name;
        SoundEffect {
            name: name.to_owned(),
            sink: None,
            audio_instance: None,
            volume: self.volume
        }
    }
}


#[derive(Clone, Debug, PartialEq)]
pub enum AudioState {
    Playing,
    Paused,
    Stopped
}
