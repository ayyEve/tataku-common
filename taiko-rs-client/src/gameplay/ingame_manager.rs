use std::{sync::{Arc, Weak}, time::Instant};

use piston::RenderArgs;
use parking_lot::Mutex;
use opengl_graphics::GlyphCache;

use taiko_rs_common::types::{Replay, Score};
use crate::game::{Audio, AudioHandle, Settings, get_font};
use crate::gameplay::*;
use crate::render::{Renderable, Rectangle, Text, Color};
use crate::{HIT_POSITION, WINDOW_SIZE, Vector2};

use super::modes::taiko::TaikoGame;

const LEAD_IN_TIME:f32 = 1000.0; // how much time should pass at beatmap start before audio begins playing (and the map "starts")
// const SV_FACTOR:f64 = 700.0; // bc sv is bonked, divide it by this amount
// const DURATION_HEIGHT:f64 = 35.0; // how tall is the duration bar
const OFFSET_DRAW_TIME:i64 = 2_000; // how long should the offset be drawn for?

// const HIT_TIMING_BAR_SIZE:Vector2 = Vector2::new(WINDOW_SIZE.x / 3.0, 30.0);
// const HIT_TIMING_BAR_POS:Vector2 = Vector2::new(WINDOW_SIZE.x / 2.0 - HIT_TIMING_BAR_SIZE.x / 2.0, WINDOW_SIZE.y - (DURATION_HEIGHT + 3.0 + HIT_TIMING_BAR_SIZE.y + 5.0));
// const HIT_TIMING_DURATION:f64 = 1_000.0; // how long should a hit timing line last
// const HIT_TIMING_FADE:f64 = 300.0; // how long to fade out for
// const HIT_TIMING_BAR_COLOR:Color = Color::new(0.0, 0.0, 0.0, 1.0); // hit timing bar color


pub struct IngameManager {
    pub score: Option<Score>,
    pub replay: Option<Replay>,

    pub started: bool,
    pub completed: bool,

    pub song_start: Instant,
    pub song: Weak<AudioHandle>,
    pub lead_in_time: f32,

    // offset things
    offset: i64,
    offset_changed_time: i64,

    pub beatmap: Beatmap,
    pub gamemode: Arc<Mutex<dyn GameMode>>,


    pub font: Arc<Mutex< GlyphCache<'static>>>,
}
impl IngameManager {
    pub fn new(beatmap: Beatmap) -> Self {

        Self {
            gamemode: Arc::new(Mutex::new(TaikoGame::new(&beatmap))),
            beatmap,

            song_start: Instant::now(),
            score: None,
            replay: None,

            song: Weak::new(), // temp until we get the audio file path
            started: false,
            completed: false,

            lead_in_time: LEAD_IN_TIME,

            offset: 0,
            offset_changed_time: 0,
            font: get_font("main"),
        }
    }

    pub fn time(&mut self) -> i64 {
        match self.song.upgrade() {
            Some(_song) => {}
            None => {
                println!("song doesnt exist at Beatmap.time()!!");
                self.song = Audio::play_song(self.beatmap.metadata.audio_filename.clone(), true);
                self.song.upgrade().unwrap().pause();
            }
        }
        self.song.upgrade().unwrap().current_time() as i64 - (self.lead_in_time as i64 + self.offset)
    }

    pub fn increment_offset(&mut self, delta:i64) {
        self.offset += delta;
        self.offset_changed_time = self.time();
    }


    // can be from either paused or new
    pub fn start(&mut self) {
        if !self.started {

            self.reset();

            match self.song.upgrade() {
                Some(song) => {
                    song.set_position(0.0);
                }
                None => {
                    self.song = Audio::play_song(self.beatmap.metadata.audio_filename.clone(), true);
                    self.song.upgrade().unwrap().pause();
                }
            }
            self.lead_in_time = LEAD_IN_TIME;
            self.song_start = Instant::now();
            // volume is set when the song is actually started (when lead_in_time is <= 0)
            self.started = true;

            return;
        } else if self.lead_in_time <= 0.0 {
            self.song.upgrade().unwrap().play();
        }
    }
    pub fn pause(&mut self) {
        self.song.upgrade().unwrap().pause();
        // is there anything else we need to do?

        // might mess with lead-in but meh
    }
    pub fn reset(&mut self) {
        let settings = Settings::get().clone();
        
        // reset song
        match self.song.upgrade().clone() {
            Some(song) => {
                song.set_position(0.0);
                song.pause();
            },
            None => {
                self.song = Audio::play_song(self.beatmap.metadata.audio_filename.clone(), true);
                let s = self.song.upgrade().unwrap();
                s.pause();
            },
        }

        self.gamemode.lock().reset(self.beatmap.clone());

        self.completed = false;
        self.started = false;
        self.lead_in_time = LEAD_IN_TIME;
        self.offset_changed_time = 0;
        self.song_start = Instant::now();

        self.score = Some(Score::new(self.beatmap.hash.clone(), settings.username.clone()));
        self.replay = Some(Replay::new());
    }


    pub fn update(&mut self) {
        // check lead-in time
        if self.lead_in_time > 0.0 {
            let elapsed = self.song_start.elapsed().as_micros() as f32 / 1000.0;
            self.song_start = Instant::now();
            self.lead_in_time -= elapsed;

            if self.lead_in_time <= 0.0 {
                let song = self.song.upgrade().unwrap();
                song.set_position(-self.lead_in_time);
                song.set_volume(Settings::get().get_music_vol());
                song.play();
                self.lead_in_time = 0.0;
            }
        }


        let m = self.gamemode.clone();
        let mut m = m.lock();
        m.update(self);
    }


    pub fn key_down(&mut self, key:piston::Key) {
        let m = self.gamemode.clone();
        let mut m = m.lock();
        m.key_down(key, self);
    }
    pub fn key_up(&mut self, key:piston::Key) {
        let m = self.gamemode.clone();
        let mut m = m.lock();
        m.key_up(key, self);
    }
    pub fn mouse_move(&mut self, pos:Vector2) {

    }
    pub fn mouse_down(&mut self, btn:piston::MouseButton) {

    }
    pub fn mouse_up(&mut self, btn:piston::MouseButton) {

    }


    pub fn draw(&mut self, args: RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        let time = self.time();
        let font = self.font.clone(); //get_font("main");

        // draw offset
        if self.offset_changed_time > 0 && time - self.offset_changed_time < OFFSET_DRAW_TIME {
            let mut offset_text = Text::new(
                Color::BLACK,
                -20.0,
                Vector2::new(0.0,0.0), // centered anyways
                32,
                format!("Offset: {}", self.offset),
                font.clone()
            );
            offset_text.center_text(Rectangle::bounds_only(Vector2::zero(), Vector2::new(WINDOW_SIZE.x , HIT_POSITION.y)));
            list.push(Box::new(offset_text));
        }


        let m = self.gamemode.clone();
        let mut m = m.lock();
        m.draw(args, self, list);
    }
}





pub trait GameMode {
    fn new(beatmap:&Beatmap) -> Self where Self: Sized;

    fn update(&mut self, manager:&mut IngameManager);
    fn draw(&mut self, args:RenderArgs, manager:&mut IngameManager, list: &mut Vec<Box<dyn Renderable>>);

    fn key_down(&mut self, key:piston::Key, manager:&mut IngameManager);
    fn key_up(&mut self, key:piston::Key, manager:&mut IngameManager);
    fn mouse_move(&mut self, pos:Vector2, manager:&mut IngameManager);
    fn mouse_down(&mut self, btn:piston::MouseButton, manager:&mut IngameManager);
    fn mouse_up(&mut self, btn:piston::MouseButton, manager:&mut IngameManager);

    fn pause(&mut self, manager:&mut IngameManager);
    fn unpause(&mut self, manager:&mut IngameManager);
    fn reset(&mut self, beatmap:Beatmap);

    fn skip_intro(&mut self) {}
}
