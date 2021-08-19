use std::{sync::{Arc, Weak}, time::Instant};

use piston::RenderArgs;
use parking_lot::Mutex;
use opengl_graphics::GlyphCache;

use taiko_rs_common::types::{KeyPress, PlayMode, Replay, ReplayFrame, Score};

use crate::gameplay::*;
use crate::{WINDOW_SIZE, Vector2};
use crate::render::{Renderable, Rectangle, Text, Color};
use crate::game::{Audio, AudioHandle, Settings, get_font};

const LEAD_IN_TIME:f32 = 1000.0; // how much time should pass at beatmap start before audio begins playing (and the map "starts")
const OFFSET_DRAW_TIME:i64 = 2_000; // how long should the offset be drawn for?


pub struct IngameManager {
    pub score: Score,
    pub replay: Replay,

    pub started: bool,
    pub completed: bool,
    pub replaying: bool,

    pub song_start: Instant,
    pub song: Weak<AudioHandle>,
    pub lead_in_time: f32,

    // offset things
    offset: i64,
    offset_changed_time: i64,

    pub beatmap: Beatmap,
    pub gamemode: Arc<Mutex<dyn GameMode>>,


    pub font: Arc<Mutex< GlyphCache<'static>>>,


    /// if in replay mode, what replay frame are we at?
    replay_frame: u64
}
impl IngameManager {
    pub fn new(beatmap: Beatmap, gamemode: Arc<Mutex<dyn GameMode>>) -> Self {
        let playmode = gamemode.lock().playmode();

        Self {
            song_start: Instant::now(),
            score: Score::new(beatmap.hash.clone(), Settings::get().username.clone(), playmode),

            replay: Replay::new(),
            gamemode,
            beatmap,

            song: Weak::new(), // temp until we get the audio file path
            started: false,
            completed: false,
            replaying: false,

            lead_in_time: LEAD_IN_TIME,

            offset: 0,
            offset_changed_time: 0,
            replay_frame: 0,
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
            }
            None => {
                self.song = Audio::play_song(self.beatmap.metadata.audio_filename.clone(), true);
                let s = self.song.upgrade().unwrap();
                s.pause();
            }
        }

        self.gamemode.lock().reset(self.beatmap.clone());

        self.completed = false;
        self.started = false;
        self.lead_in_time = LEAD_IN_TIME;
        self.offset_changed_time = 0;
        self.song_start = Instant::now();
        self.score = Score::new(self.beatmap.hash.clone(), settings.username.clone(), self.gamemode.lock().playmode());
        self.replay_frame = 0;

        // only reset the replay if we arent replaying
        if !self.replaying {
            self.replay = Replay::new();
        }
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
        let time = self.time();

        if self.replaying {
            let m = self.gamemode.clone();
            let mut m = m.lock();

            // read any frames that need to be read
            loop {
                if self.replay_frame as usize >= self.replay.frames.len() {break}
                
                let (frame_time, frame) = self.replay.frames[self.replay_frame as usize];
                if frame_time > time {break}
                m.handle_replay_frame(frame, self);

                // this should be handled by the gamemode
                // match pressed {
                //     KeyPress::LeftKat => {
                //         let mut hit = HalfCircle::new(
                //             Color::BLUE,
                //             HIT_POSITION,
                //             1.0,
                //             HIT_AREA_RADIUS,
                //             true
                //         );
                //         hit.set_lifetime(DRUM_LIFETIME_TIME);
                //         self.render_queue.push(Box::new(hit));
                //     },
                //     KeyPress::LeftDon => {
                //         let mut hit = HalfCircle::new(
                //             Color::RED,
                //             HIT_POSITION,
                //             1.0,
                //             HIT_AREA_RADIUS,
                //             true
                //         );
                //         hit.set_lifetime(DRUM_LIFETIME_TIME);
                //         self.render_queue.push(Box::new(hit));
                //     },
                //     KeyPress::RightDon => {
                //         let mut hit = HalfCircle::new(
                //             Color::RED,
                //             HIT_POSITION,
                //             1.0,
                //             HIT_AREA_RADIUS,
                //             false
                //         );
                //         hit.set_lifetime(DRUM_LIFETIME_TIME);
                //         self.render_queue.push(Box::new(hit));
                //     },
                //     KeyPress::RightKat => {
                //         let mut hit = HalfCircle::new(
                //             Color::BLUE,
                //             HIT_POSITION,
                //             1.0,
                //             HIT_AREA_RADIUS,
                //             false
                //         );
                //         hit.set_lifetime(DRUM_LIFETIME_TIME);
                //         self.render_queue.push(Box::new(hit));
                //     },
                // }

                self.replay_frame += 1;
            }
        }

        let m = self.gamemode.clone();
        let mut m = m.lock();
        m.update(self);
    }


    pub fn key_down(&mut self, key:piston::Key) {
        let m = self.gamemode.clone();
        let mut m = m.lock();

        if self.replaying {
            // check replay-only keys
            if key == piston::Key::Escape {
                self.started = false;
                self.completed = true;
                return;
            }
        }

        // skip intro
        if key == piston::Key::Space {
            m.skip_intro(self);
        }

        m.key_down(key, self);
    }
    pub fn key_up(&mut self, key:piston::Key) {
        let m = self.gamemode.clone();
        let mut m = m.lock();
        m.key_up(key, self);
    }
    pub fn mouse_move(&mut self, pos:Vector2) {
        let m = self.gamemode.clone();
        let mut m = m.lock();
        m.mouse_move(pos, self);
    }
    pub fn mouse_down(&mut self, btn:piston::MouseButton) {
        let m = self.gamemode.clone();
        let mut m = m.lock();
        m.mouse_down(btn, self);
    }
    pub fn mouse_up(&mut self, btn:piston::MouseButton) {
        let m = self.gamemode.clone();
        let mut m = m.lock();
        m.mouse_up(btn, self);
    }


    pub fn draw(&mut self, args: RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        let time = self.time();
        let font = self.font.clone();

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
            offset_text.center_text(Rectangle::bounds_only(Vector2::zero(), Vector2::new(WINDOW_SIZE.x , WINDOW_SIZE.y * 2.0/3.0)));
            list.push(Box::new(offset_text));
        }


        let m = self.gamemode.clone();
        let mut m = m.lock();
        m.draw(args, self, list);
    }
}





pub trait GameMode {
    fn new(beatmap:&Beatmap) -> Self where Self: Sized;
    fn playmode(&self) -> PlayMode;
    // fn hit(&mut self, key:KeyPress, manager:&mut IngameManager);

    fn handle_replay_frame(&mut self, frame:ReplayFrame, manager:&mut IngameManager);

    fn update(&mut self, manager:&mut IngameManager);
    fn draw(&mut self, args:RenderArgs, manager:&mut IngameManager, list: &mut Vec<Box<dyn Renderable>>);

    fn key_down(&mut self, key:piston::Key, manager:&mut IngameManager);
    fn key_up(&mut self, key:piston::Key, manager:&mut IngameManager);
    fn mouse_move(&mut self, _pos:Vector2, _manager:&mut IngameManager) {}
    fn mouse_down(&mut self, _btn:piston::MouseButton, _manager:&mut IngameManager) {}
    fn mouse_up(&mut self, _btn:piston::MouseButton, _manager:&mut IngameManager) {}


    fn skip_intro(&mut self, manager: &mut IngameManager);
    fn pause(&mut self, _manager:&mut IngameManager) {}
    fn unpause(&mut self, _manager:&mut IngameManager) {}
    fn reset(&mut self, beatmap:Beatmap);
}
