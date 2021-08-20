use std::{sync::{Arc, Weak}, time::Instant};

use ayyeve_piston_ui::render::Border;
use piston::RenderArgs;
use parking_lot::Mutex;
use opengl_graphics::GlyphCache;

use taiko_rs_common::types::{PlayMode, Replay, ReplayFrame, Score};

use crate::{gameplay::*, helpers::visibility_bg};
use crate::{WINDOW_SIZE, Vector2};
use crate::render::{Renderable, Rectangle, Text, Color};
use crate::game::{Audio, AudioHandle, Settings, get_font};

const LEAD_IN_TIME:f32 = 1000.0; // how much time should pass at beatmap start before audio begins playing (and the map "starts")
const OFFSET_DRAW_TIME:i64 = 2_000; // how long should the offset be drawn for?
const DURATION_HEIGHT:f64 = 35.0; // how tall is the duration bar


const HIT_TIMING_BAR_SIZE:Vector2 = Vector2::new(WINDOW_SIZE.x / 3.0, 30.0);
const HIT_TIMING_BAR_POS:Vector2 = Vector2::new(WINDOW_SIZE.x / 2.0 - HIT_TIMING_BAR_SIZE.x / 2.0, WINDOW_SIZE.y - (DURATION_HEIGHT + 3.0 + HIT_TIMING_BAR_SIZE.y + 5.0));
const HIT_TIMING_DURATION:f64 = 1_000.0; // how long should a hit timing line last
const HIT_TIMING_FADE:f64 = 300.0; // how long to fade out for
const HIT_TIMING_BAR_COLOR:Color = Color::new(0.0, 0.0, 0.0, 1.0); // hit timing bar color


pub struct IngameManager {
    pub beatmap: Beatmap,
    pub gamemode: Arc<Mutex<dyn GameMode>>,

    pub score: Score,
    pub replay: Replay,

    pub started: bool,
    pub completed: bool,
    pub replaying: bool,
    pub end_time: f64,

    pub song_start: Instant,
    pub song: Weak<AudioHandle>,
    pub lead_in_time: f32,

    // offset things
    offset: i64,
    offset_changed_time: i64,

    /// (map.time, note.time - hit.time)
    pub hitbar_timings: Vec<(i64, i64)>,

    // draw helpers
    pub font: Arc<Mutex< GlyphCache<'static>>>,
    combo_text_bounds: Rectangle,
    timing_bar_things: (Vec<(f64,Color)>, (f64,Color)),


    /// if in replay mode, what replay frame are we at?
    replay_frame: u64
}
impl IngameManager {
    pub fn new(beatmap: Beatmap, gamemode: Arc<Mutex<dyn GameMode>>) -> Self {
        let lock = gamemode.clone();
        let lock = lock.lock();
        let playmode = lock.playmode();

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
            end_time: lock.end_time(),

            offset: 0,
            offset_changed_time: 0,
            replay_frame: 0,


            font: get_font("main"),
            combo_text_bounds: lock.combo_bounds(),
            timing_bar_things: lock.timing_bar_things(),
            hitbar_timings: Vec::new()
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

        let mut lock = self.gamemode.lock();
        lock.reset(self.beatmap.clone());

        self.completed = false;
        self.started = false;
        self.lead_in_time = LEAD_IN_TIME;
        self.offset_changed_time = 0;
        self.song_start = Instant::now();
        self.score = Score::new(self.beatmap.hash.clone(), settings.username.clone(), lock.playmode());
        self.replay_frame = 0;


        self.combo_text_bounds = lock.combo_bounds();
        self.timing_bar_things = lock.timing_bar_things();
        self.hitbar_timings = Vec::new();



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

        // update hit timings bar
        self.hitbar_timings.retain(|(hit_time, _)| {time - hit_time < HIT_TIMING_DURATION as i64});

        // update gamemode
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


        // gamemode things

        // score bg
        list.push(visibility_bg(
            Vector2::new(args.window_size[0] - 200.0, 10.0),
            Vector2::new(180.0, 75.0 - 10.0)
        ));
        // score text
        list.push(Box::new(Text::new(
            Color::BLACK,
            0.0,
            Vector2::new(args.window_size[0] - 200.0, 40.0),
            30,
            crate::format(self.score.score),
            font.clone()
        )));

        // acc text
        list.push(Box::new(Text::new(
            Color::BLACK,
            0.0,
            Vector2::new(args.window_size[0] - 200.0, 70.0),
            30,
            format!("{:.2}%", self.score.acc()*100.0),
            font.clone()
        )));

        // combo text
        let mut combo_text = Text::new(
            Color::WHITE,
            0.0,
            Vector2::zero(),
            30,
            crate::format(self.score.combo),
            font.clone()
        );
        combo_text.center_text(self.combo_text_bounds);
        list.push(Box::new(combo_text));


        // duration bar
        // duration remaining
        list.push(Box::new(Rectangle::new(
            Color::new(0.4, 0.4, 0.4, 0.5),
            1.0,
            Vector2::new(0.0, args.window_size[1] - (DURATION_HEIGHT + 3.0)),
            Vector2::new(args.window_size[0], DURATION_HEIGHT),
            Some(Border::new(Color::BLACK, 1.8))
        )));
        // fill
        list.push(Box::new(Rectangle::new(
            [0.4,0.4,0.4,1.0].into(),
            2.0,
            Vector2::new(0.0, args.window_size[1] - (DURATION_HEIGHT + 3.0)),
            Vector2::new(args.window_size[0] * (time as f64/self.end_time), DURATION_HEIGHT),
            None
        )));


        // draw hit timings bar
        // draw hit timing colors below the bar
        let (windows, (miss, miss_color)) = &self.timing_bar_things;
        
        //draw miss window first
        list.push(Box::new(Rectangle::new(
            *miss_color,
            17.1,
            Vector2::new((WINDOW_SIZE.x-HIT_TIMING_BAR_SIZE.x)/2.0, HIT_TIMING_BAR_POS.y),
            Vector2::new(HIT_TIMING_BAR_SIZE.x, HIT_TIMING_BAR_SIZE.y),
            None // for now
        )));

        // draw other windows
        for (window, color) in windows {
            let width = window / miss * HIT_TIMING_BAR_SIZE.x;
            list.push(Box::new(Rectangle::new(
                *color,
                17.0,
                Vector2::new((WINDOW_SIZE.x - width)/2.0, HIT_TIMING_BAR_POS.y),
                Vector2::new(width, HIT_TIMING_BAR_SIZE.y),
                None // for now
            )));
        }
       

        // draw hit timings
        let time = time as f64;
        for (hit_time, diff) in self.hitbar_timings.as_slice() {
            let hit_time = hit_time.clone() as f64;
            let mut diff = diff.clone() as f64;
            if diff < 0.0 {
                diff = diff.max(-miss);
            } else {
                diff = diff.min(*miss);
            }

            let pos = diff / miss * (HIT_TIMING_BAR_SIZE.x / 2.0);

            // draw diff line
            let diff = time - hit_time;
            let alpha = if diff > HIT_TIMING_DURATION - HIT_TIMING_FADE {
                1.0 - (diff - (HIT_TIMING_DURATION - HIT_TIMING_FADE)) / HIT_TIMING_FADE
            } else {1.0};

            let mut c = HIT_TIMING_BAR_COLOR;
            c.a = alpha as f32;
            list.push(Box::new(Rectangle::new(
                c,
                10.0,
                Vector2::new(WINDOW_SIZE.x  / 2.0 + pos, HIT_TIMING_BAR_POS.y),
                Vector2::new(2.0, HIT_TIMING_BAR_SIZE.y),
                None // for now
            )));
        }

        // draw gamemode
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


    fn end_time(&self) -> f64;


    fn combo_bounds(&self) -> Rectangle;

    /// f64 is hitwindow, color is color for that window. last is miss hitwindow
    fn timing_bar_things(&self) -> (Vec<(f64,Color)>, (f64,Color));
}
