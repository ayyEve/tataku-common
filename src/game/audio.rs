use std::time::SystemTime;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::{io::{Cursor, Read}, fs::File, path::Path};

use rodio::{Decoder, OutputStream, Sink, OutputStreamHandle};
use crate::game::Settings;

type AudioData = Cursor<Vec<u8>>;

const SOUND_LIST:[&'static str; 4] = [
    "audio/kat.wav",
    "audio/don.wav",
    "audio/bigdon.wav",
    "audio/bigkat.wav"
];

lazy_static::lazy_static! {
    static ref SOUNDS: HashMap<String, AudioData> = {
        let mut sounds:HashMap<String, AudioData> = HashMap::new();

        for sound in SOUND_LIST.iter() {
            let sound_name = Path::new(sound).file_stem().unwrap().to_str().unwrap();

            let mut data = Vec::new();
            File::open(sound).unwrap().read_to_end(&mut data).unwrap();
            let cur = Cursor::new(data);

            sounds.insert(sound_name.to_owned(), cur);
        }

        sounds
    };
    static ref STREAM: Arc<Mutex<Option<OutputStreamHandle>>> = Arc::new(Mutex::new(None));
}

pub struct Audio;
impl Audio {
    /// play a sound effect. uses settings.get_effect_vol() for volume
    pub fn play(name:&str) {
        let sink = Audio::load_sink(name);
        sink.set_volume(Settings::get().get_effect_vol());
        sink.play();
        sink.detach();
    }

    pub fn load_sink(name:&str) -> Sink {

        // check if we don't have the audio pre-loaded
        if !SOUNDS.contains_key(name) {
            if !Path::new(name).exists() {
                println!("[Audio] File not found! {}. No audio will be played", name);
                return Sink::new_idle().0;
            }

            // load from file manually
            let mut data = Vec::new();
            File::open(name).unwrap().read_to_end(&mut data).unwrap();
            let cur = Cursor::new(data);

            let lock = STREAM.lock().unwrap();
            let stream_handle = lock.as_ref().unwrap();

            // Decode that sound file into a source
            let source = Decoder::new(cur).unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();
            sink.append(source);
            sink.pause();
            return sink;
        }

        // otherwise load from cache
        let cur = SOUNDS.get(name).unwrap().clone();
        let lock = STREAM.lock().unwrap();
        let stream_handle = lock.as_ref().unwrap();

        // Decode that sound file into a source
        let source = Decoder::new(cur).unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.append(source);
        sink.pause();
        sink
    }

    pub fn from_raw(data:Vec<u8>) -> Sink {
    
        let cur = Cursor::new(data);

        let lock = STREAM.lock().unwrap();
        let stream_handle = lock.as_ref().unwrap();

        // Decode that sound file into a source
        let source = Decoder::new(cur).unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.append(source);
        sink.pause();
        return sink;
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
    sink: Option<Sink>,
    // when was the effect started?
    start_time: SystemTime,
    /// if sound is paused, add the current duration to this
    previous_duration: u64,
    // what state is this sound effect in?
    pub state: AudioState,

    // effect volume
    volume:f32
}
impl SoundEffect {
    pub fn new(sound:&str) -> SoundEffect {
        SoundEffect {
            name: sound.to_owned(),
            // none as it is loaded on play (since state is stopped)
            sink: None, //Audio::load_sink(sound), 
            start_time: SystemTime::now(),
            previous_duration: 0,
            state: AudioState::Stopped,
            volume: 1.0
        }
    }
    pub fn new_empty() -> SoundEffect {
        SoundEffect {
            name: "".to_owned(),
            sink: None,
            start_time: SystemTime::now(),
            previous_duration: 0,
            state: AudioState::Stopped,
            volume: 1.0
        }
    }

    pub fn duration(&self) -> u64 {
        match &self.state {
            AudioState::Playing => {
                self.start_time.elapsed().unwrap().as_millis() as u64 + self.previous_duration
            },
            AudioState::Paused => {
                self.previous_duration
            },
            AudioState::Stopped => 0,
        }
    }

    pub fn set_volume(&mut self, vol:f32) {
        self.volume = vol;
        match &self.state {
            AudioState::Stopped => {},
            _ => {
                self.sink.as_ref().unwrap().set_volume(vol);
            }
        }
    }

    pub fn play(&mut self) {
        match &self.state {
            AudioState::Playing => {}, // do nothing, its already playing
            AudioState::Paused => { // resume the sink
                self.start_time = SystemTime::now();
                self.sink.as_ref().unwrap().play();
            },
            AudioState::Stopped => { // reload the sink and play it
                // also reset previous duration
                self.previous_duration = 0;

                self.start_time = SystemTime::now();
                let sink = Audio::load_sink(&self.name);
                sink.set_volume(self.volume);
                sink.play();
                self.sink = Some(sink);
            },
        }
        
        self.state = AudioState::Playing;
    }
    pub fn pause(&mut self) {
        match &self.state {
            AudioState::Playing => {
                self.previous_duration = self.duration();
                self.sink.as_ref().unwrap().pause();
                self.state = AudioState::Paused;
            },
            _ => {}
        }
    }
    pub fn stop(&mut self) {
        match &self.state {
            AudioState::Stopped => {}, // do nothing, its already stopped
            _ => {
                self.state = AudioState::Stopped; 
                self.previous_duration = 0;
                self.sink.as_ref().unwrap().stop();
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
            start_time: SystemTime::now(),
            previous_duration: 0,
            state: AudioState::Stopped,
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
