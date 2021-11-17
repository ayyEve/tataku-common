use std::time::Instant;
use std::collections::HashMap;

use bass::prelude::*;
use piston::RenderArgs;
use opengl_graphics::GlyphCache;

use crate::game::managers::ModManager;
use crate::{beatmaps::Beatmap, errors::TaikoError};
use crate::game::{Audio, BackgroundGameSettings, Settings};
use crate::beatmaps::osu::hitobject_defs::HitSamples;
use crate::helpers::centered_text_helper::CenteredTextHelper;
use taiko_rs_common::types::{PlayMode, Replay, ReplayFrame, Score};
use crate::{Vector2, sync::*, helpers::{visibility_bg, io::exists}};
use crate::beatmaps::common::{BeatmapMeta, TaikoRsBeatmap, TimingPoint};
use crate::render::{Renderable, Rectangle, Text, Color, Border, fonts::get_font};

const LEAD_IN_TIME:f32 = 1000.0; // how much time should pass at beatmap start before audio begins playing (and the map "starts")
const OFFSET_DRAW_TIME:f32 = 2_000.0; // how long should the offset be drawn for?
pub const DURATION_HEIGHT:f64 = 35.0; // how tall is the duration bar


const HIT_TIMING_BAR_SIZE:Vector2 = Vector2::new(300.0, 30.0);
const HIT_TIMING_BAR_POS:Vector2 = Vector2::new(200.0 - HIT_TIMING_BAR_SIZE.x / 2.0, -(DURATION_HEIGHT + 3.0 + HIT_TIMING_BAR_SIZE.y + 5.0));
const HIT_TIMING_DURATION:f32 = 1_000.0; // how long should a hit timing line last
const HIT_TIMING_FADE:f32 = 300.0; // how long to fade out for
const HIT_TIMING_BAR_COLOR:Color = Color::new(0.0, 0.0, 0.0, 1.0); // hit timing bar color


pub struct IngameManager {
    pub beatmap: Beatmap,
    pub metadata: BeatmapMeta,
    pub gamemode: Box<dyn GameMode>,
    pub current_mods: Arc<ModManager>,

    pub score: Score,
    pub replay: Replay,

    pub started: bool,
    pub completed: bool,
    pub replaying: bool,
    /// is this playing in the background of the main menu?
    pub menu_background: bool,
    pub end_time: f32,

    pub lead_in_time: f32,
    pub lead_in_timer: Instant,

    pub timing_points: Vec<TimingPoint>,
    pub timing_point_index: usize,
    pub song: StreamChannel,

    pub hitsound_cache: HashMap<String, Option<SampleChannel>>,

    // offset things
    offset: CenteredTextHelper<f32>,
    global_offset: CenteredTextHelper<f32>,

    /// (map.time, note.time - hit.time)
    pub hitbar_timings: Vec<(f32, f32)>,

    // draw helpers
    pub font: Arc<Mutex<GlyphCache<'static>>>,
    combo_text_bounds: Rectangle,
    timing_bar_things: (Vec<(f32,Color)>, (f32,Color)),

    /// if in replay mode, what replay frame are we at?
    replay_frame: u64,

    background_game_settings: BackgroundGameSettings,
}
impl IngameManager {
    pub fn new(beatmap: Beatmap, gamemode: Box<dyn GameMode>) -> Self {
        let playmode = gamemode.playmode();
        let metadata = beatmap.get_beatmap_meta();

        let settings = Settings::get_mut().clone();
        let timing_points = beatmap.get_timing_points();
        let font = get_font("main");
        let hitsound_cache = HashMap::new();

        let current_mods = Arc::new(ModManager::get().clone());

        let mut score =  Score::new(beatmap.hash().clone(), settings.username.clone(), playmode);
        score.speed = current_mods.speed;

        Self {
            metadata,
            timing_points,
            hitsound_cache,
            current_mods,

            lead_in_timer: Instant::now(),
            score,

            replay: Replay::new(),
            beatmap,

            song: Audio::get_song().unwrap_or(create_empty_stream()), // temp until we get the audio file path
            started: false,
            completed: false,
            replaying: false,
            menu_background: false,

            lead_in_time: LEAD_IN_TIME,
            end_time: gamemode.end_time(),

            offset: CenteredTextHelper::new("Offset", 0.0, OFFSET_DRAW_TIME, -20.0, font.clone()),
            global_offset: CenteredTextHelper::new("Global Offset", 0.0, OFFSET_DRAW_TIME, -20.0, font.clone()),
            
            replay_frame: 0,
            timing_point_index: 0,

            font,
            combo_text_bounds: gamemode.combo_bounds(),
            timing_bar_things: gamemode.timing_bar_things(),
            hitbar_timings: Vec::new(),

            background_game_settings: settings.background_game_settings.clone(),

            gamemode,
        }
    }

    pub fn should_save_score(&self) -> bool {
        let should = !(self.replaying || self.current_mods.autoplay);
        println!("should?: {}", should);
        should
    }


    pub fn current_timing_point(&self) -> TimingPoint {
        self.timing_points[self.timing_point_index]
    }

    pub fn time(&mut self) -> f32 {
        // let t = match (self.song.upgrade(), Audio::get_song_raw()) {
        //     (None, Some((_, song))) => {
        //         match song.upgrade() {
        //             Some(s) => {
        //                 self.song = song;
        //                 s.current_time()
        //             }
        //             None => {
        //                 println!("song doesnt exist at Beatmap.time()!!");
        //                 self.song = Audio::play_song(self.metadata.audio_filename.clone(), true, 0.0);
        //                 self.song.upgrade().unwrap().pause();
        //                 0.0
        //             }
        //         }
        //     },
        //     (None, None) => {
        //         println!("song doesnt exist at Beatmap.time()!!");
        //         self.song = Audio::play_song(self.metadata.audio_filename.clone(), true, 0.0);
        //         self.song.upgrade().unwrap().pause();
        //         0.0
        //     }
        //     (Some(song), _) => song.current_time(),
        // };
        let t = self.song.get_position().unwrap() as f32;

        // println!("time: {}", t);

        t - (self.lead_in_time + self.offset.value + self.global_offset.value)
    }

    pub fn increment_offset(&mut self, delta:f32) {
        let time = self.time();
        let new_val = self.offset.value + delta;
        self.offset.set_value(new_val, time);
    }

    /// locks settings
    pub fn increment_global_offset(&mut self, delta:f32) {
        let mut settings = Settings::get_mut();
        settings.global_offset += delta;

        let time = self.time();
        self.global_offset.set_value(settings.global_offset, time);
    }


    // is this game pausable
    pub fn can_pause(&mut self) -> bool {
        !(self.current_mods.autoplay || self.replaying)
    }

    // can be from either paused or new
    pub fn start(&mut self) {

        if !self.started {
            self.reset();

            if self.menu_background {
                // dont reset the song, and dont do lead in
                self.lead_in_time = 0.0;
            } else {
                self.song.set_position(0.0).unwrap();
                self.song.pause().unwrap();
                
                self.lead_in_timer = Instant::now();
                self.lead_in_time = LEAD_IN_TIME;

                if self.replaying {
                    self.song.set_rate(self.replay.speed).unwrap();
                }
            }

            // volume is set when the song is actually started (when lead_in_time is <= 0)
            self.started = true;

        } else if self.lead_in_time <= 0.0 {
            // if this is the menu, dont do anything
            if self.menu_background {return}

            self.song.play(true).unwrap();

            // // needed because if paused for a while it can crash
            // match self.song.upgrade() {
            //     Some(song) => song.play(),
            //     None => self.song = Audio::play_song(self.metadata.audio_filename.clone(), true, 0.0),
            // }
        }
    }
    pub fn pause(&mut self) {
        self.song.pause().unwrap();
        // is there anything else we need to do?

        // might mess with lead-in but meh
    }
    pub fn reset(&mut self) {
        let settings = Settings::get();
        
        self.gamemode.reset(&self.beatmap);

        if self.menu_background {
            self.background_game_settings = settings.background_game_settings.clone();
            self.gamemode.apply_auto(&self.background_game_settings)
        } else {
            // reset song
            // match self.song.upgrade() {
            //     Some(song) => {
            //         song.set_position(0.0);
            //         song.pause();
            //         song.set_playback_speed(self.current_mods.speed as f64);
            //     }
            //     None => {
            //         while let None = self.song.upgrade() {
            //             self.song = Audio::play_song(self.metadata.audio_filename.clone(), true, 0.0);
            //         }
            //         let song = self.song.upgrade().unwrap();
            //         song.set_playback_speed(self.current_mods.speed as f64);
            //         song.pause();
            //     }
            // }
            self.song.set_rate(self.current_mods.speed).unwrap();
            self.song.set_position(0.0).unwrap();
            self.song.pause().unwrap();
        }

        self.completed = false;
        self.started = false;
        self.lead_in_time = LEAD_IN_TIME;
        // self.offset_changed_time = 0.0;
        self.lead_in_timer = Instant::now();
        self.score = Score::new(self.beatmap.hash(), settings.username.clone(), self.gamemode.playmode());
        self.replay_frame = 0;
        self.timing_point_index = 0;

        self.combo_text_bounds = self.gamemode.combo_bounds();
        self.timing_bar_things = self.gamemode.timing_bar_things();
        self.hitbar_timings = Vec::new();

        
        if self.replaying {
            // if we're replaying, make sure we're using the score's speed
            self.song.set_rate(self.replay.speed).unwrap();
        } else {
            // only reset the replay if we arent replaying
            self.replay = Replay::new();
            self.score.speed = self.current_mods.speed;
        }
    }


    pub fn update(&mut self) {
        // check lead-in time
        if self.lead_in_time > 0.0 {
            let elapsed = self.lead_in_timer.elapsed().as_micros() as f32 / 1000.0;
            self.lead_in_timer = Instant::now();
            self.lead_in_time -= elapsed * if self.replaying {self.replay.speed} else {self.current_mods.speed};

            if self.lead_in_time <= 0.0 {
                // let song = self.song.upgrade().unwrap();
                self.song.set_position(-self.lead_in_time as f64).unwrap();
                self.song.set_volume(Settings::get().get_music_vol()).unwrap();
                if self.replaying {
                    self.song.set_rate(self.replay.speed).unwrap();
                } else {
                    self.song.set_rate(self.current_mods.speed).unwrap();
                }
                
                self.song.play(true).unwrap();
                self.lead_in_time = 0.0;
            }
        }
        let time = self.time();

        // check timing point
        let timing_points = &self.timing_points;
        if self.timing_point_index + 1 < timing_points.len() && timing_points[self.timing_point_index + 1].time <= time {
            self.timing_point_index += 1;
        }

        let mut gamemode = std::mem::take(&mut self.gamemode);

        // read inputs from replay if replaying
        if self.replaying && !self.current_mods.autoplay {

            // read any frames that need to be read
            loop {
                if self.replay_frame as usize >= self.replay.frames.len() {break}
                
                let (frame_time, frame) = self.replay.frames[self.replay_frame as usize];
                if frame_time > time {break}

                gamemode.handle_replay_frame(frame, frame_time, self);
                
                self.replay_frame += 1;
            }
        }

        // update hit timings bar
        self.hitbar_timings.retain(|(hit_time, _)| {time - hit_time < HIT_TIMING_DURATION});

        // update gamemode
        gamemode.update(self, time);

        // put it back
        self.gamemode = gamemode;
    }


    pub fn play_note_sound(&mut self, note_time:f32, note_hitsound: u8, note_hitsamples:HitSamples) {
        let timing_point = self.beatmap.control_point_at(note_time);
        
        let mut play_normal = true; //(note_hitsound & 1) > 0; // 0: Normal
        let play_whistle = (note_hitsound & 2) > 0; // 1: Whistle
        let play_finish = (note_hitsound & 4) > 0; // 2: Finish
        let play_clap = (note_hitsound & 8) > 0; // 3: Clap

        // get volume
        let mut vol = (if note_hitsamples.volume == 0 {timing_point.volume} else {note_hitsamples.volume} as f32 / 100.0) * Settings::get_mut().get_effect_vol();
        if self.menu_background {vol *= self.background_game_settings.hitsound_volume};


        // https://osu.ppy.sh/wiki/en/osu%21_File_Formats/Osu_%28file_format%29#hitsounds

        // normalSet and additionSet can be any of the following:
        // 0: No custom sample set
        // For normal sounds, the set is determined by the timing point's sample set.
        // For additions, the set is determined by the normal sound's sample set.
        // 1: Normal set
        // 2: Soft set
        // 3: Drum set

        // The filename is <sampleSet>-hit<hitSound><index>.wav, where:

        // sampleSet is normal, soft, or drum, determined by either normalSet or additionSet depending on which hitsound is playing
        const SAMPLE_SETS:&[&str] = &["normal", "normal", "soft", "drum"];
        // hitSound is normal, whistle, finish, or clap
        // index is the same index as above, except it is not written if the value is 0 or 1

        // (filename, index)
        let mut play_list = Vec::new();

        // if the hitsound is being overridden
        if let Some(name) = note_hitsamples.filename {
            if name.len() > 0 {
                println!("got custom sound: {}", name);
                if exists(format!("resources/audio/{}", name)) {
                    play_normal = (note_hitsound & 1) > 0;
                    play_list.push((name, 0))
                } else {
                    println!("doesnt exist");
                }
            }
        }

        if play_normal {
            let sample_set = SAMPLE_SETS[note_hitsamples.addition_set as usize];
            let hitsound = format!("{}-hitnormal.wav", sample_set);
            let index = note_hitsamples.index;
            // if sample_set == 0 {sample_set = timing_point.sample_set}
            // if index == 1 {} //idk wtf 

            play_list.push((hitsound, index))
        }

        if play_whistle {
            let sample_set = SAMPLE_SETS[note_hitsamples.addition_set as usize];
            let hitsound = format!("{}-hitwhistle.wav", sample_set);
            let index = note_hitsamples.index;
            // if sample_set == 0 {sample_set = timing_point.sample_set}
            // if index == 1 {} //idk wtf 

            play_list.push((hitsound, index))
        }
        if play_finish {
            let sample_set = SAMPLE_SETS[note_hitsamples.addition_set as usize];
            let hitsound = format!("{}-hitfinish.wav", sample_set);
            let index = note_hitsamples.index;
            // if sample_set == 0 {sample_set = timing_point.sample_set}
            // if index == 1 {} //idk wtf 

            play_list.push((hitsound, index))
        }
        if play_clap {
            let sample_set = SAMPLE_SETS[note_hitsamples.addition_set as usize];
            let hitsound = format!("{}-hitclap.wav", sample_set);
            let index = note_hitsamples.index;
            // if sample_set == 0 {sample_set = timing_point.sample_set}
            // if index == 1 {} //idk wtf 

            play_list.push((hitsound, index))
        }


        // The sound file is loaded from the first of the following directories that contains a matching filename:
        // Beatmap, if index is not 0
        // Skin, with the index removed
        // Default osu! resources, with the index removed
        // When filename is given, no addition sounds will be played, and this file in the beatmap directory is played instead.

        // println!("{}, {} | {}", timing_point.volume, note_hitsamples.volume, );


        for (sound_file, _index) in play_list.iter() {
            if !self.hitsound_cache.contains_key(sound_file) {
                println!("not cached");

                let sound = Audio::load(format!("resources/audio/{}", sound_file));
                if let Err(e) = &sound {
                    println!("error loading: {:?}", e);
                }
                
                self.hitsound_cache.insert(sound_file.clone(), sound.ok());
            }

            if let Some(sound) = self.hitsound_cache.get(sound_file).unwrap() {
                sound.set_volume(vol).unwrap();
                sound.set_position(0.0).unwrap();
                sound.play(true).unwrap();
            }
        }
    }

    pub fn combo_break(&mut self) {
        if self.score.combo >= 20 && !self.menu_background {
            // play hitsound
            Audio::play_preloaded("combobreak").unwrap();
        }

        // reset combo to 0
        self.score.combo = 0;
    }

    pub fn key_down(&mut self, key:piston::Key, mods: ayyeve_piston_ui::menu::KeyModifiers) {
        if (self.replaying || self.current_mods.autoplay) && !self.menu_background {
            // check replay-only keys
            if key == piston::Key::Escape {
                println!("poo");
                self.started = false;
                self.completed = true;
                return;
            }
        }

        let mut gamemode = std::mem::take(&mut self.gamemode);

        // skip intro
        if key == piston::Key::Space {
            gamemode.skip_intro(self);
        }

        // check for offset changing keys
        {
            let settings = Settings::get_mut();
            if mods.shift {
                let mut t = 0.0;
                if key == settings.key_offset_up {t = 5.0}
                if key == settings.key_offset_down {t = -5.0}
                drop(settings);

                if t != 0.0 {
                    self.increment_global_offset(t);
                }
            } else {
                if key == settings.key_offset_up {self.increment_offset(5.0)}
                if key == settings.key_offset_down {self.increment_offset(-5.0)}
            }
        }


        gamemode.key_down(key, self);
        self.gamemode = gamemode;
    }
    pub fn key_up(&mut self, key:piston::Key) {
        let mut gamemode = std::mem::take(&mut self.gamemode);
        gamemode.key_up(key, self);
        self.gamemode = gamemode;
    }
    pub fn mouse_move(&mut self, pos:Vector2) {
        let mut gamemode = std::mem::take(&mut self.gamemode);
        gamemode.mouse_move(pos, self);
        self.gamemode = gamemode;
    }
    pub fn mouse_down(&mut self, btn:piston::MouseButton) {
        let mut gamemode = std::mem::take(&mut self.gamemode);
        gamemode.mouse_down(btn, self);
        self.gamemode = gamemode;
    }
    pub fn mouse_up(&mut self, btn:piston::MouseButton) {
        let mut gamemode = std::mem::take(&mut self.gamemode);
        gamemode.mouse_up(btn, self);
        self.gamemode = gamemode;
    }

    pub fn mouse_scroll(&mut self, delta:f64) {
        let mut gamemode = std::mem::take(&mut self.gamemode);
        gamemode.mouse_scroll(delta, self);
        self.gamemode = gamemode;
    }

    pub fn draw(&mut self, args: RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        let time = self.time();
        let font = self.font.clone();
        let window_size:Vector2 = args.window_size.into();

        // draw gamemode
        let mut gamemode = std::mem::take(&mut self.gamemode);
        gamemode.draw(args, self, list);
        self.gamemode = gamemode;


        // draw center texts
        self.offset.draw(time, list);
        self.global_offset.draw(time, list);


        if self.menu_background {return}
        // gamemode things

        // score bg
        list.push(visibility_bg(
            Vector2::new(window_size.x - 200.0, 10.0),
            Vector2::new(180.0, 75.0 - 10.0),
            1.0
        ));
        // score text
        list.push(Box::new(Text::new(
            Color::BLACK,
            0.0,
            Vector2::new(window_size.x - 200.0, 40.0),
            30,
            crate::format(self.score.score),
            font.clone()
        )));

        // acc text
        list.push(Box::new(Text::new(
            Color::BLACK,
            0.0,
            Vector2::new(window_size.x - 200.0, 70.0),
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
            Vector2::new(0.0, window_size.y - (DURATION_HEIGHT + 3.0)),
            Vector2::new(window_size.x, DURATION_HEIGHT),
            Some(Border::new(Color::BLACK, 1.8))
        )));
        // fill
        list.push(Box::new(Rectangle::new(
            [0.4,0.4,0.4,1.0].into(),
            2.0,
            Vector2::new(0.0, window_size.y - (DURATION_HEIGHT + 3.0)),
            Vector2::new(window_size.x * (time/self.end_time) as f64, DURATION_HEIGHT),
            None
        )));


        // draw hit timings bar
        // draw hit timing colors below the bar
        let (windows, (miss, miss_color)) = &self.timing_bar_things;
        
        //draw miss window first
        list.push(Box::new(Rectangle::new(
            *miss_color,
            17.1,
            Vector2::new((window_size.x-HIT_TIMING_BAR_SIZE.x)/2.0, window_size.y + HIT_TIMING_BAR_POS.y),
            Vector2::new(HIT_TIMING_BAR_SIZE.x, HIT_TIMING_BAR_SIZE.y),
            None // for now
        )));

        // draw other hit windows
        for (window, color) in windows {
            let width = (window / miss) as f64 * HIT_TIMING_BAR_SIZE.x;
            list.push(Box::new(Rectangle::new(
                *color,
                17.0,
                Vector2::new((window_size.x - width)/2.0, window_size.y + HIT_TIMING_BAR_POS.y),
                Vector2::new(width, HIT_TIMING_BAR_SIZE.y),
                None // for now
            )));
        }
       

        // draw hit timings
        let time = time;
        for (hit_time, diff) in self.hitbar_timings.as_slice() {
            let hit_time = hit_time.clone();
            let mut diff = diff.clone();
            if diff < 0.0 {
                diff = diff.max(-miss);
            } else {
                diff = diff.min(*miss);
            }

            let pos = (diff / miss) as f64 * (HIT_TIMING_BAR_SIZE.x / 2.0);

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
                Vector2::new(window_size.x / 2.0 + pos, window_size.y + HIT_TIMING_BAR_POS.y),
                Vector2::new(2.0, HIT_TIMING_BAR_SIZE.y),
                None // for now
            )));
        }

    }
}
impl Default for IngameManager {
    fn default() -> Self {
        Self { 
            song: create_empty_stream(), 
            font: get_font("main"), 
            combo_text_bounds: Rectangle::bounds_only(Vector2::zero(), Vector2::zero()), 
            timing_bar_things: (Vec::new(), (0.0, Color::WHITE)),
            
            beatmap: Default::default(),
            metadata: Default::default(),
            gamemode: Default::default(),
            current_mods: Default::default(),
            score: Default::default(),
            replay: Default::default(),
            started: Default::default(),
            completed: Default::default(),
            replaying: Default::default(),
            menu_background: Default::default(),
            end_time: Default::default(),
            lead_in_time: Default::default(),
            lead_in_timer: Instant::now(),
            timing_points: Default::default(),
            timing_point_index: Default::default(),
            hitsound_cache: Default::default(),
            offset: Default::default(),
            global_offset: Default::default(),
            hitbar_timings: Default::default(),
            replay_frame: Default::default(),
            background_game_settings: Default::default(), 
        }
    }
}



pub trait GameMode {
    fn new(beatmap:&Beatmap) -> Result<Self, TaikoError> where Self: Sized;

    fn playmode(&self) -> PlayMode;
    fn end_time(&self) -> f32;
    fn combo_bounds(&self) -> Rectangle;
    /// f64 is hitwindow, color is color for that window. last is miss hitwindow
    fn timing_bar_things(&self) -> (Vec<(f32,Color)>, (f32,Color));
    /// convert mouse pos to mode's playfield coords
    // fn scale_mouse_pos(&self, mouse_pos:Vector2) -> Vector2 {mouse_pos}

    fn handle_replay_frame(&mut self, frame:ReplayFrame, time:f32, manager:&mut IngameManager);

    fn update(&mut self, manager:&mut IngameManager, time: f32);
    fn draw(&mut self, args:RenderArgs, manager:&mut IngameManager, list: &mut Vec<Box<dyn Renderable>>);

    fn key_down(&mut self, key:piston::Key, manager:&mut IngameManager);
    fn key_up(&mut self, key:piston::Key, manager:&mut IngameManager);
    fn mouse_move(&mut self, _pos:Vector2, _manager:&mut IngameManager) {}
    fn mouse_down(&mut self, _btn:piston::MouseButton, _manager:&mut IngameManager) {}
    fn mouse_up(&mut self, _btn:piston::MouseButton, _manager:&mut IngameManager) {}
    fn mouse_scroll(&mut self, _delta:f64, _manager:&mut IngameManager) {}

    fn apply_auto(&mut self, settings: &BackgroundGameSettings);


    fn skip_intro(&mut self, manager: &mut IngameManager);
    fn pause(&mut self, _manager:&mut IngameManager) {}
    fn unpause(&mut self, _manager:&mut IngameManager) {}
    fn reset(&mut self, beatmap:&Beatmap);
}
impl Default for Box<dyn GameMode> {
    fn default() -> Self {
        Box::new(NoMode::new(&Default::default()).unwrap())
    }
}

// needed for std::mem::take/swap
struct NoMode {}
impl GameMode for NoMode {
    fn new(_:&Beatmap) -> Result<Self, TaikoError> where Self: Sized {Ok(Self {})}
    fn playmode(&self) -> PlayMode {PlayMode::Standard}
    fn end_time(&self) -> f32 {0.0}
    fn combo_bounds(&self) -> Rectangle {Rectangle::bounds_only(Vector2::zero(), Vector2::zero())}
    fn timing_bar_things(&self) -> (Vec<(f32,Color)>, (f32,Color)) {(Vec::new(), (0.0, Color::WHITE))}

    fn handle_replay_frame(&mut self, _:ReplayFrame, _:f32, _:&mut IngameManager) {}
    fn update(&mut self, _:&mut IngameManager, _: f32) {}
    fn draw(&mut self, _:RenderArgs, _:&mut IngameManager, _: &mut Vec<Box<dyn Renderable>>) {}
    fn key_down(&mut self, _:piston::Key, _:&mut IngameManager) {}
    fn key_up(&mut self, _:piston::Key, _:&mut IngameManager) {}
    fn apply_auto(&mut self, _: &BackgroundGameSettings) {}
    fn skip_intro(&mut self, _: &mut IngameManager) {}
    fn reset(&mut self, _:&Beatmap) {}
}

// struct HitsoundManager {
//     sounds: HashMap<String, Option<Sound>>,
//     /// sound, index
//     beatmap_sounds: HashMap<String, HashMap<u8, Sound>>
// }
// impl HitsoundManager {

// }

lazy_static::lazy_static! {
    static ref EMPTY_STREAM:StreamChannel = {
        // wave file bytes with ~1 sample
        StreamChannel::create_from_memory(vec![0x52,0x49,0x46,0x46,0x28,0x00,0x00,0x00,0x57,0x41,0x56,0x45,0x66,0x6D,0x74,0x20,0x10,0x00,0x00,0x00,0x01,0x00,0x02,0x00,0x44,0xAC,0x00,0x00,0x88,0x58,0x01,0x00,0x02,0x00,0x08,0x00,0x64,0x61,0x74,0x61,0x04,0x00,0x00,0x00,0x80,0x80,0x80,0x80], 0i32).expect("error creating empty StreamChannel")
    };
}
fn create_empty_stream() -> StreamChannel {
    EMPTY_STREAM.clone()
}
