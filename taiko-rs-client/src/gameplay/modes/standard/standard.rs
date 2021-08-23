use ayyeve_piston_ui::render::*;
use piston::RenderArgs;
use taiko_rs_common::types::KeyPress;
use taiko_rs_common::types::ReplayFrame;
use taiko_rs_common::types::ScoreHit;
use taiko_rs_common::types::PlayMode;

use crate::game::Audio;
use crate::game::Settings;
use crate::gameplay::SliderDef;
use crate::gameplay::SpinnerDef;
use crate::gameplay::map_difficulty_range;
use crate::{WINDOW_SIZE, Vector2};
use crate::gameplay::{GameMode, Beatmap, IngameManager, TimingPoint};

use super::*;


pub const NOTE_RADIUS:f64 = 32.0;
pub const HIT_AREA_RADIUS:f64 = NOTE_RADIUS * 1.3;
pub const HIT_POSITION:Vector2 = Vector2::new(180.0, 200.0);
pub const PLAYFIELD_RADIUS:f64 = NOTE_RADIUS * 2.0; // actually height, oops

pub const BAR_COLOR:Color = Color::new(0.0, 0.0, 0.0, 1.0); // timing bar color
const BAR_WIDTH:f64 = 4.0; // how wide is a timing bar
const BAR_SPACING:f64 = 4.0; // how many beats between timing bars

const SV_FACTOR:f64 = 700.0; // bc sv is bonked, divide it by this amount

/// how long should the drum buttons last for?
const DRUM_LIFETIME_TIME:u64 = 100;


pub struct StandardGame {
    // lists
    pub notes: Vec<Box<dyn StandardHitObject>>,
    
    /// where to start checking notes from
    note_index: usize,
    timing_point_index: usize,

    // hit timing bar stuff
    hitwindow_300: f64,
    hitwindow_100: f64,
    hitwindow_miss: f64,

    end_time: f64,

    render_queue: Vec<Box<HalfCircle>>,
}
impl StandardGame {
    pub fn next_note(&mut self) {self.note_index += 1}
}

impl GameMode for StandardGame {
    fn playmode(&self) -> PlayMode {PlayMode::Standard}
    fn end_time(&self) -> f64 {self.end_time}
    fn new(beatmap:&Beatmap) -> Self {
        let mut s = Self {
            notes: Vec::new(),
            note_index: 0,

            timing_point_index: 0,
            end_time: 0.0,

            hitwindow_100: 0.0,
            hitwindow_300: 0.0,
            hitwindow_miss: 0.0,

            render_queue: Vec::new()
        };

        // // add notes
        // for note in beatmap.notes.iter() {
        //     let hit_type = if (note.hitsound & (2 | 8)) > 0 {super::HitType::Kat} else {super::HitType::Don};
        //     let finisher = (note.hitsound & 4) > 0;

        //     s.notes.push(Box::new(TaikoNote::new(
        //         note.time as u64,
        //         hit_type,
        //         finisher,
        //         1.0
        //     )));
        // }
        // for slider in beatmap.sliders.iter() {
        //     let SliderDef {time, slides, length, ..} = slider.to_owned();
        //     let time = time as u64;
        //     let finisher = (slider.hitsound & 4) > 0;

        //     let l = (length * 1.4) * slides as f64;
        //     let v2 = 100.0 * (beatmap.metadata.slider_multiplier as f64 * 1.4);
        //     let bl = beatmap.beat_length_at(time as f64, true);
        //     let end_time = time + (l / v2 * bl) as u64;
            
        //     // convert vars
        //     let v = beatmap.slider_velocity_at(time as u64);
        //     let bl = beatmap.beat_length_at(time as f64, beatmap.metadata.beatmap_version < 8.0);
        //     let skip_period = (bl / beatmap.metadata.slider_tick_rate as f64).min((end_time - time) as f64 / slides as f64);

        //     if skip_period > 0.0 && beatmap.metadata.mode != PlayMode::Taiko && l / v * 1000.0 < 2.0 * bl {
        //         let mut i = 0;
        //         let mut j = time as f64;

        //         // load sounds
        //         // let sound_list_raw = if let Some(list) = split.next() {list.split("|")} else {"".split("")};

        //         // when loading, if unified just have it as sound_types with 1 index
        //         let mut sound_types:Vec<(HitType, bool)> = Vec::new();

        //         // for i in sound_list_raw {
        //         //     if let Ok(hitsound) = i.parse::<u32>() {
        //         //         let hit_type = if (hitsound & (2 | 8)) > 0 {super::HitType::Kat} else {super::HitType::Don};
        //         //         let finisher = (hitsound & 4) > 0;
        //         //         sound_types.push((hit_type, finisher));
        //         //     }
        //         // }
                
        //         let unified_sound_addition = sound_types.len() == 0;
        //         if unified_sound_addition {
        //             sound_types.push((HitType::Don, false));
        //         }

        //         //TODO: could this be turned into a for i in (x..y).step(n) ?
        //         loop {
        //             let sound_type = sound_types[i];

        //             let note = TaikoNote::new(
        //                 j as u64,
        //                 sound_type.0,
        //                 sound_type.1,
        //                 1.0
        //             );
        //             s.notes.push(Box::new(note));

        //             if !unified_sound_addition {i = (i + 1) % sound_types.len()}

        //             j += skip_period;
        //             if !(j < end_time as f64 + skip_period / 8.0) {break}
        //         }
        //     } else {
        //         let slider = TaikoSlider::new(time, end_time, finisher, 1.0);
        //         s.notes.push(Box::new(slider));
        //     }
        // }
        // for spinner in beatmap.spinners.iter() {
        //     let SpinnerDef {time, end_time, ..} = spinner;

        //     let length = end_time - time;
        //     let diff_map = map_difficulty_range(beatmap.metadata.od as f64, 3.0, 5.0, 7.5);
        //     let hits_required:u16 = ((length / 1000.0 * diff_map) * 1.65).max(1.0) as u16; // ((this.Length / 1000.0 * this.MapDifficultyRange(od, 3.0, 5.0, 7.5)) * 1.65).max(1.0)

        //     s.notes.push(Box::new(TaikoSpinner::new(*time as u64, *end_time as u64, 1.0, hits_required)));
        // }

        s.notes.sort_by(|a, b|a.time().cmp(&b.time()));
        s.end_time = s.notes.iter().last().unwrap().time() as f64;

        s
    }

    fn handle_replay_frame(&mut self, frame:ReplayFrame, manager:&mut IngameManager) {
        let time = manager.time() as f64;
        if !manager.replaying {
            manager.replay.frames.push((time as i64, frame.clone()));
        }
        let key = match frame {
            ReplayFrame::Press(k) => k,
            ReplayFrame::Release(k) => k
        };

    }


    fn update(&mut self, manager:&mut IngameManager) {
        // get the current time
        let time = manager.time();

        // update notes
        for note in self.notes.iter_mut() {note.update(time)}

        // if theres no more notes to hit, show score screen
        if self.note_index >= self.notes.len() {
            manager.completed = true;
            return;
        }

        // since some std maps are non-linear (2b),
        // we need to check all notes up until a certain criteria 
        // TODO! figure out this criteria

        // // check if we missed the current note
        // if (self.notes[self.note_index].end_time(self.hitwindow_miss) as i64) < time {
        //     if self.notes[self.note_index].causes_miss() {
        //         // need to set these manually instead of score.hit_miss,
        //         // since we dont want to add anything to the hit error list
        //         let s = &mut manager.score;
        //         s.xmiss += 1;
        //         s.combo = 0;
        //     }
        //     self.next_note();
        // }
        
        let timing_points = &manager.beatmap.timing_points;
        // check timing point
        if self.timing_point_index + 1 < timing_points.len() && timing_points[self.timing_point_index + 1].time <= time as f64 {
            self.timing_point_index += 1;
        }
    }
    fn draw(&mut self, args:RenderArgs, manager:&mut IngameManager, list:&mut Vec<Box<dyn Renderable>>) {
        for i in self.render_queue.iter() {
            list.push(i.clone());
        }
        self.render_queue.clear();

        // draw the playfield
        let playfield = Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            f64::MAX-4.0,
            Vector2::new(0.0, HIT_POSITION.y - (PLAYFIELD_RADIUS + 2.0)),
            Vector2::new(args.window_size[0], (PLAYFIELD_RADIUS+2.0) * 2.0),
            if manager.beatmap.timing_points[self.timing_point_index].kiai {
                Some(Border::new(Color::YELLOW, 2.0))
            } else {None}
        );
        list.push(Box::new(playfield));

        // draw the hit area
        list.push(Box::new(Circle::new(
            Color::BLACK,
            f64::MAX,
            HIT_POSITION,
            HIT_AREA_RADIUS + 2.0
        )));

        // draw notes
        for note in self.notes.iter_mut() {list.extend(note.draw(args))}
    }


    fn key_down(&mut self, key:piston::Key, manager:&mut IngameManager) {
        let settings = Settings::get().taiko_settings;

        if key == settings.left_kat {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::LeftKat), manager);
        }
        if key == settings.left_don {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::LeftDon), manager);
        }
        if key == settings.right_don {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::RightDon), manager);
        }
        if key == settings.right_kat {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::RightKat), manager);
        }
    }
    fn key_up(&mut self, _key:piston::Key, _manager:&mut IngameManager) {}

    fn reset(&mut self, beatmap:Beatmap) {
        let settings = Settings::get().taiko_settings;
        
        for note in self.notes.as_mut_slice() {
            note.reset();
        }
        
        self.note_index = 0;
        self.timing_point_index = 0;

        let od = beatmap.metadata.od as f64;
        // setup hitwindows
        self.hitwindow_miss = map_difficulty_range(od, 135.0, 95.0, 70.0);
        self.hitwindow_100 = map_difficulty_range(od, 120.0, 80.0, 50.0);
        self.hitwindow_300 = map_difficulty_range(od, 50.0, 35.0, 20.0);

    }



    fn skip_intro(&mut self, manager: &mut IngameManager) {
        if self.note_index > 0 {return}

        let x_needed = WINDOW_SIZE.x;
        let mut time = manager.time();

        // loop {
        //     let mut found = false;
        //     for note in self.notes.iter() {if note.x_at(time) <= x_needed {found = true; break}}
        //     if found {break}
        //     time += 1;
        // }

        let mut time = time as f32;
        if manager.lead_in_time > 0.0 {
            if time > manager.lead_in_time {
                time -= manager.lead_in_time - 0.01;
                manager.lead_in_time = 0.01;
            }
        }

        manager.song.upgrade().unwrap().set_position(time);
    }


    fn timing_bar_things(&self) -> (Vec<(f64,Color)>, (f64,Color)) {
        (vec![
            (self.hitwindow_100, [0.3411, 0.8901, 0.0745, 1.0].into()),
            (self.hitwindow_300, [0.1960, 0.7372, 0.9058, 1.0].into()),
        ], (self.hitwindow_miss, [0.8549, 0.6823, 0.2745, 1.0].into()))
    }

    fn combo_bounds(&self) -> Rectangle {
        let size = Vector2::new(0.0, 30.0);
        Rectangle::bounds_only(
            Vector2::new(0.0, WINDOW_SIZE.y - size.y),
            size
        )
    }
}

