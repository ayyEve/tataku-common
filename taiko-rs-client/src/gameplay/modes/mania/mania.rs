use std::u8;

use piston::RenderArgs;
use ayyeve_piston_ui::render::*;
use taiko_rs_common::types::{KeyPress, ReplayFrame, PlayMode};

// use crate::game::Audio;
use crate::game::Settings;
use crate::{WINDOW_SIZE, Vector2};
use crate::gameplay::{HoldDef, NoteType};
use super::{ManiaHold, ManiaNote, ManiaHitObject};
use crate::gameplay::{GameMode, Beatmap, IngameManager, TimingPoint, map_difficulty_range};


pub const COLUMN_WIDTH: f64 = 100.0;
pub const NOTE_SIZE:Vector2 = Vector2::new(COLUMN_WIDTH, 30.0);
pub const NOTE_BORDER_SIZE:f64 = 1.4;
pub const HIT_Y:f64 = WINDOW_SIZE.y - 100.0;


pub const BAR_COLOR:Color = Color::new(0.0, 0.0, 0.0, 1.0); // timing bar color
const BAR_HEIGHT:f64 = 4.0; // how tall is a timing bar
const BAR_SPACING:f64 = 4.0; // how many beats between timing bars
const BAR_DEPTH:f64 = -90.0; // how many beats between timing bars

const SV_FACTOR:f64 = 700.0; // bc sv is bonked, divide it by this amount
const COLUMN_COUNT:u8 = 4; //TODO!!

pub struct ManiaGame {
    // lists
    columns: Vec<Vec<Box<dyn ManiaHitObject>>>,
    timing_bars: Vec<TimingBar>,
    // list indices
    timing_point_index: usize,
    column_indices: Vec<usize>,
    /// true if held
    column_states: Vec<bool>,

    // hit timing bar stuff
    hitwindow_300: f64,
    hitwindow_100: f64,
    hitwindow_miss: f64,

    end_time: f64,
    column_count: u8,
}
impl ManiaGame {
    /// get the x_pos for `col`
    pub fn col_pos(&self, col:u8) -> f64{
        let total_width = self.column_count as f64 * COLUMN_WIDTH;
        let x_offset = (WINDOW_SIZE.x - total_width) / 2.0;

        x_offset + col as f64 * COLUMN_WIDTH
    }

    pub fn get_color(&self, _col:u8) -> Color {
        Color::WHITE
    }

    fn next_note(&mut self, col:usize) {
        (*self.column_indices.get_mut(col).unwrap()) += 1;
    }

    fn set_sv(&mut self, sv:f64) {
        let sv = sv / SV_FACTOR;

        for col in self.columns.iter_mut() {
            for note in col.iter_mut() {
                note.set_sv(sv);
            }
        }
        for bar in self.timing_bars.iter_mut() {
            bar.set_sv(sv);
        }
    }
}

impl GameMode for ManiaGame {
    fn playmode(&self) -> PlayMode {PlayMode::Mania}
    fn end_time(&self) -> f64 {self.end_time}

    fn new(beatmap:&Beatmap) -> Self {
        let mut s = Self {
            columns: Vec::new(),
            column_indices:Vec::new(),
            column_states: Vec::new(),

            timing_bars: Vec::new(),
            timing_point_index: 0,
            end_time: 0.0,

            hitwindow_100: 0.0,
            hitwindow_300: 0.0,
            hitwindow_miss: 0.0,

            column_count: COLUMN_COUNT
        };

        // init defaults for the columsn
        for _col in 0..s.column_count {
            s.columns.push(Vec::new());
            s.column_indices.push(0);
            s.column_states.push(false);
        }

        // add notes
        for note in beatmap.notes.iter() {
            if beatmap.metadata.mode == PlayMode::Mania {
                let column = (note.pos.x * s.column_count as f64 / 512.0).floor() as u8;
                let x = s.col_pos(column);
                s.columns[column as usize].push(Box::new(ManiaNote::new(
                    note.time as u64,
                    x
                )));
            }
        }
        for hold in beatmap.holds.iter() {
            let HoldDef {pos, time, end_time, ..} = hold.to_owned();
            let time = time as u64;

            let column = (pos.x * s.column_count as f64 / 512.0).floor() as u8;
            let x = s.col_pos(column);
            s.columns[column as usize].push(Box::new(ManiaHold::new(
                time as u64,
                end_time as u64,
                x
            )));
        }
        
        for _slider in beatmap.sliders.iter() {
            // let SliderDef {pos, time, slides, length, ..} = slider.to_owned();
            // let time = time as u64;

            // let l = (length * 1.4) * slides as f64;
            // let v2 = 100.0 * (beatmap.metadata.slider_multiplier as f64 * 1.4);
            // let bl = beatmap.beat_length_at(time as f64, true);
            // let end_time = time + (l / v2 * bl) as u64;
    
            // let column = (pos.x * s.column_count as f64 / 512.0).floor() as u8;
            // let x = s.col_pos(column);
            // s.columns[column as usize].push(Box::new(ManiaHold::new(
            //     time as u64,
            //     end_time as u64,
            //     x
            // )));
        }
        for _spinner in beatmap.spinners.iter() {
            // let SpinnerDef {time, end_time, ..} = spinner;
            //TODO
        }

        for col in s.columns.iter_mut() {
            col.sort_by(|a, b|a.time().cmp(&b.time()));
            s.end_time = s.end_time.max(col.iter().last().unwrap().time() as f64);
        }
        
        s
    }

    fn handle_replay_frame(&mut self, frame:ReplayFrame, manager:&mut IngameManager) {
        let time = manager.time() as f64;
        if !manager.replaying {
            manager.replay.frames.push((time as i64, frame));
        }

        match frame {
            ReplayFrame::Press(key) => {
                let col:usize = match key {
                    KeyPress::Mania1 => 0,
                    KeyPress::Mania2 => 1,
                    KeyPress::Mania3 => 2,
                    KeyPress::Mania4 => 3,
                    KeyPress::Mania5 => 4,
                    KeyPress::Mania6 => 5,
                    KeyPress::Mania7 => 6,
                    KeyPress::Mania8 => 7,
                    KeyPress::Mania9 => 8,
                    _ => return
                };
                // let hit_type:HitType = key.into();
                // let mut sound = match hit_type {HitType::Don => "don", HitType::Kat => "kat"};
                // let hit_volume = Settings::get().get_effect_vol() * (manager.beatmap.timing_points[self.timing_point_index].volume as f32 / 100.0);

                // if theres no more notes to hit, return after playing the sound
                if self.column_indices[col] >= self.columns[col].len() {
                    // let a = Audio::play_preloaded(sound);
                    // a.upgrade().unwrap().set_volume(hit_volume);
                    return;
                }
                let note = &mut self.columns[col][self.column_indices[col]];
                let note_time = note.time() as f64;
                *self.column_states.get_mut(col).unwrap() = true;

                let diff = (time - note_time).abs();
                // normal note
                if diff < self.hitwindow_300 {
                    note.hit(time);

                    manager.score.hit300(time as u64, note_time as u64);
                    manager.hitbar_timings.push((time as i64, (time - note_time) as i64));
                    // Audio::play_preloaded(sound);
                    if note.note_type() != NoteType::Hold {
                        self.next_note(col);
                    }
                } else if diff < self.hitwindow_100 {
                    note.hit(time as f64);

                    manager.score.hit100(time as u64, note_time as u64);
                    manager.hitbar_timings.push((time as i64, (time - note_time) as i64));
                    // Audio::play_preloaded(sound);
                    //TODO: indicate this was a bad hit

                    if note.note_type() != NoteType::Hold {
                        self.next_note(col);
                    }
                } else if diff < self.hitwindow_miss { // too early, miss
                    note.miss(time);

                    manager.score.hit_miss(time as u64, note_time as u64);
                    manager.hitbar_timings.push((time as i64, (time - note_time) as i64));
                    if note.note_type() != NoteType::Hold {
                        self.next_note(col);
                    }
                    // Audio::play_preloaded(sound);
                    //TODO: play miss sound
                    //TODO: indicate this was a miss
                } else { // way too early, ignore
                    // play sound
                    // Audio::play_preloaded(sound);
                }
            
            }
            ReplayFrame::Release(key) => {
                let col:usize = match key {
                    KeyPress::Mania1 => 0,
                    KeyPress::Mania2 => 1,
                    KeyPress::Mania3 => 2,
                    KeyPress::Mania4 => 3,
                    KeyPress::Mania5 => 4,
                    KeyPress::Mania6 => 5,
                    KeyPress::Mania7 => 6,
                    KeyPress::Mania8 => 7,
                    KeyPress::Mania9 => 8,
                    _ => return
                };
                *self.column_states.get_mut(col).unwrap() = false;
                if self.column_indices[col] >= self.columns[col].len() {return}

                let note = &mut self.columns[col][self.column_indices[col]];
                if time < note.time() as f64 - self.hitwindow_miss 
                || time > note.end_time(self.hitwindow_miss) as f64 {return}
                note.release(time);

                if note.note_type() == NoteType::Hold {
                    let note_time = note.end_time(0.0) as f64;
                    let diff = (time - note_time as f64).abs();
                    // normal note
                    if diff < self.hitwindow_300 {
                        manager.score.hit300(time as u64, note_time as u64);
                        manager.hitbar_timings.push((time as i64, (time - note_time) as i64));
                        // Audio::play_preloaded(sound);

                        self.next_note(col);
                    } else if diff < self.hitwindow_100 {
                        manager.score.hit100(time as u64, note_time as u64);
                        manager.hitbar_timings.push((time as i64, (time - note_time) as i64));
                        // Audio::play_preloaded(sound);
                        //TODO: indicate this was a bad hit

                        self.next_note(col);
                    } else if diff < self.hitwindow_miss { // too early, miss
                        manager.score.hit_miss(time as u64, note_time as u64);
                        manager.hitbar_timings.push((time as i64, (time - note_time) as i64));
                        // Audio::play_preloaded(sound);
                        //TODO: play miss sound
                        //TODO: indicate this was a miss
                        self.next_note(col);
                    }
                }
                
                // self.columns[col][self.column_indices[col]].release(time);
            }
        }
    }

    fn key_down(&mut self, key:piston::Key, manager:&mut IngameManager) {
        let settings = Settings::get();
        let mut game_key = KeyPress::RightDon;
        if key == settings.left_kat {
            game_key = KeyPress::Mania1;
            // self.hit(, manager);
        }
        if key == settings.left_don {
            game_key = KeyPress::Mania2;
            // self.hit(KeyPress::Mania2, manager);
        }
        if key == settings.right_don {
            game_key = KeyPress::Mania3;
            // self.hit(KeyPress::Mania3, manager);
        }
        if key == settings.right_kat {
            game_key = KeyPress::Mania4;
            // self.hit(KeyPress::Mania4, manager);
        }

        self.handle_replay_frame(ReplayFrame::Press(game_key), manager);
    }
    fn key_up(&mut self, key:piston::Key, manager:&mut IngameManager) {
        let settings = Settings::get();
        let mut game_key = KeyPress::RightDon;
        if key == settings.left_kat {
            game_key = KeyPress::Mania1;
        }
        if key == settings.left_don {
            game_key = KeyPress::Mania2;
        }
        if key == settings.right_don {
            game_key = KeyPress::Mania3;
        }
        if key == settings.right_kat {
            game_key = KeyPress::Mania4;
        }

        self.handle_replay_frame(ReplayFrame::Release(game_key), manager);
    }

    fn reset(&mut self, beatmap:Beatmap) {
        for col in self.columns.iter_mut() {
            for note in col.iter_mut() {
                note.reset();
            }
        }
        
        self.timing_point_index = 0;

        let od = beatmap.metadata.od as f64;
        // setup hitwindows
        self.hitwindow_miss = map_difficulty_range(od, 135.0, 95.0, 70.0);
        self.hitwindow_100 = map_difficulty_range(od, 120.0, 80.0, 50.0);
        self.hitwindow_300 = map_difficulty_range(od, 50.0, 35.0, 20.0);

        // setup timing bars
        //TODO: it would be cool if we didnt actually need timing bar objects, and could just draw them
        if self.timing_bars.len() == 0 {
            // load timing bars
            let parent_tps = beatmap.timing_points.iter().filter(|t|!t.is_inherited()).collect::<Vec<&TimingPoint>>();
            let mut time = parent_tps[0].time;
            let mut tp_index = 0;
            let step = beatmap.beat_length_at(time, false);
            time %= step; // get the earliest bar line possible

            let bar_width = self.column_count as f64 * COLUMN_WIDTH;
            let x = (WINDOW_SIZE.x - bar_width) / 2.0;

            loop {
                // if theres a bpm change, adjust the current time to that of the bpm change
                let next_bar_time = beatmap.beat_length_at(time, false) * BAR_SPACING; // bar spacing is actually the timing point measure

                // edge case for aspire maps
                if next_bar_time.is_nan() || next_bar_time == 0.0 {
                    break;
                }

                // add timing bar at current time
                self.timing_bars.push(TimingBar::new(time as u64, bar_width, x));

                if tp_index < parent_tps.len() && parent_tps[tp_index].time <= time + next_bar_time {
                    time = parent_tps[tp_index].time;
                    tp_index += 1;
                    continue;
                }

                // why isnt this accounting for bpm changes? because the bpm change doesnt allways happen inline with the bar idiot
                time += next_bar_time;
                if time >= self.end_time || time.is_nan() {break}
            }

            println!("created {} timing bars", self.timing_bars.len());
        }
        

        let sv = beatmap.slider_velocity_at(0);
        self.set_sv(sv);
    }


    fn update(&mut self, manager:&mut IngameManager) {
        // get the current time
        let time = manager.time();

        // update notes
        for col in self.columns.iter_mut() {
            for note in col.iter_mut() {note.update(time)}
        }

        // show score screen if map is over
        if time >= self.end_time as i64 {
            manager.completed = true;
            return;
        }

        // check if we missed the current note
        for col in 0..self.column_count as usize {
            if self.column_indices[col] >= self.columns[col].len() {continue}
            let note = &self.columns[col][self.column_indices[col]];
            if (note.end_time(self.hitwindow_miss) as i64) <= time {
                // need to set these manually instead of score.hit_miss,
                // since we dont want to add anything to the hit error list
                let s = &mut manager.score;
                s.xmiss += 1;
                s.combo = 0;
                
                self.next_note(col);
            }
        }
        
        // TODO: might move tbs to a (time, speed) tuple
        for tb in self.timing_bars.iter_mut() {tb.update(time as f64)}

        let timing_points = &manager.beatmap.timing_points;
        // check timing point
        if self.timing_point_index + 1 < timing_points.len() && timing_points[self.timing_point_index + 1].time <= time as f64 {
            self.timing_point_index += 1;
            // let tp = &timing_points[self.timing_point_index];
            let sv = manager.beatmap.slider_velocity_at(time as u64);
            self.set_sv(sv);
        }
    }
    fn draw(&mut self, args:RenderArgs, _manager:&mut IngameManager, list:&mut Vec<Box<dyn Renderable>>) {

        // draw columns
        for col in 0..self.column_count {
            let x = self.col_pos(col);
            list.push(Box::new(Rectangle::new(
                self.get_color(col),
                1000.0,
                Vector2::new(x, 0.0),
                Vector2::new(COLUMN_WIDTH, WINDOW_SIZE.y),
                Some(Border::new(Color::GREEN, 1.2))
            )));

            // hit area for this col
            list.push(Box::new(Rectangle::new(
                Color::TRANSPARENT_WHITE,
                -100.0,
                Vector2::new(x, HIT_Y),
                NOTE_SIZE,
                Some(Border::new(Color::RED, 2.0))
            )));
            // draw button state for this col
            if self.column_states[col as usize] {
                list.push(Box::new(Rectangle::new(
                    self.get_color(col),
                    -110.0,
                    Vector2::new(x, HIT_Y),
                    NOTE_SIZE,
                    Some(Border::new(Color::RED, 2.0))
                )));
            }
        }
        
        // column background
        list.push(Box::new(Rectangle::new(
            Color::new(0.0, 0.0, 0.0, 0.8),
            1000.0,
            Vector2::new(self.col_pos(0), 0.0),
            Vector2::new(self.col_pos(self.column_count) - self.col_pos(0), WINDOW_SIZE.y),
            Some(Border::new(Color::GREEN, 1.2))
        )));

        // draw notes
        for col in self.columns.iter_mut() {
            for note in col.iter_mut() {list.extend(note.draw(args))}
        }
        // draw timing lines
        for tb in self.timing_bars.iter_mut() {list.extend(tb.draw(args))}
    }

    fn skip_intro(&mut self, manager: &mut IngameManager) {
        let y_needed = 0.0;
        let mut time = manager.time() as f64;

        loop {
            let mut found = false;

            for col in self.columns.iter_mut() {
                for note in col.iter_mut() {
                    if note.y_at(time) <= y_needed {found = true; break}
                }
            }
            if found {break}
            time += 1.0;
        }

        let mut time = time as f32;
        if manager.lead_in_time > 0.0 {
            if time > manager.lead_in_time {
                time -= manager.lead_in_time - 0.01;
                manager.lead_in_time = 0.01;
            }
        }

        manager.song.upgrade().unwrap().set_position(time);
    }

    fn combo_bounds(&self) -> Rectangle {
        Rectangle::bounds_only(
            Vector2::new(0.0, WINDOW_SIZE.y * (1.0/3.0)),
            Vector2::new(WINDOW_SIZE.x, 30.0)
        )
    }

    fn timing_bar_things(&self) -> (Vec<(f64,Color)>, (f64,Color)) {
        (vec![
            (self.hitwindow_100, [0.3411, 0.8901, 0.0745, 1.0].into()),
            (self.hitwindow_300, [0.1960, 0.7372, 0.9058, 1.0].into()),
        ], (self.hitwindow_miss, [0.8549, 0.6823, 0.2745, 1.0].into()))
    }
}


// timing bar struct
//TODO: might be able to reduce this to a (time, speed) and just calc pos on draw
#[derive(Copy, Clone, Debug)]
struct TimingBar {
    time: u64,
    speed: f64,
    pos: Vector2,
    size: Vector2
}
impl TimingBar {
    pub fn new(time:u64, width:f64, x:f64) -> TimingBar {
        TimingBar {
            time, 
            size: Vector2::new(width, BAR_HEIGHT),
            speed: 1.0,
            pos: Vector2::new(x, 0.0),
        }
    }

    pub fn set_sv(&mut self, sv:f64) {
        self.speed = sv;
    }

    pub fn update(&mut self, time:f64) {
        self.pos.y = (HIT_Y + NOTE_SIZE.y-self.size.y) - (self.time as f64 - time as f64) * self.speed;
        // self.pos = HIT_POSITION + Vector2::new(( - BAR_WIDTH / 2.0, -PLAYFIELD_RADIUS);
    }

    fn draw(&mut self, _args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut renderables: Vec<Box<dyn Renderable>> = Vec::new();
        if self.pos.y < 0.0 || self.pos.y > WINDOW_SIZE.y {return renderables}

        renderables.push(Box::new(Rectangle::new(
            BAR_COLOR,
            BAR_DEPTH,
            self.pos,
            self.size,
            None
        )));

        renderables
    }
}
