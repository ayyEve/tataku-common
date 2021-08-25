use core::f32;

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
use crate::helpers::slider::get_curve;
use crate::{WINDOW_SIZE, Vector2};
use crate::gameplay::{GameMode, Beatmap, IngameManager, TimingPoint};

use super::*;


pub const NOTE_RADIUS:f64 = 32.0;
pub const HIT_AREA_RADIUS:f64 = NOTE_RADIUS * 1.3;
pub const HIT_POSITION:Vector2 = Vector2::new(180.0, 200.0);
pub const PLAYFIELD_RADIUS:f64 = NOTE_RADIUS * 2.0; // actually height, oops

pub const BAR_COLOR:Color = Color::new(0.0, 0.0, 0.0, 1.0); // timing bar color
const BAR_WIDTH:f64 = 4.0; // how wide is a timing bar

const SV_FACTOR:f32 = 700.0; // bc sv is bonked, divide it by this amount

/// how long should the drum buttons last for?
const DRUM_LIFETIME_TIME:u64 = 100;


pub struct StandardGame {
    // lists
    pub notes: Vec<Box<dyn StandardHitObject>>,
    
    /// where to start checking notes from
    note_index: usize,
    timing_point_index: usize,

    // hit timing bar stuff
    hitwindow_300: f32,
    hitwindow_100: f32,
    hitwindow_miss: f32,

    end_time: f32,

    render_queue: Vec<Box<HalfCircle>>,
}
impl StandardGame {
    pub fn next_note(&mut self) {self.note_index += 1}
}

impl GameMode for StandardGame {
    fn playmode(&self) -> PlayMode {PlayMode::Standard}
    fn end_time(&self) -> f32 {self.end_time}
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

        // let ar = beatmap.metadata.
        let ar = 7.0;
        let cs = 5.0;

        // add notes
        for note in beatmap.notes.iter() {
            s.notes.push(Box::new(StandardNote::new(
                note.clone(),
                ar,
                cs,
                Color::BLUE,
                1
            )));
        }
        for slider in beatmap.sliders.iter() {
            let curve = get_curve(slider, beatmap);
            s.notes.push(Box::new(StandardSlider::new(
                slider.clone(),
                curve,
                Color::BLUE,
                1
            )))
        }
        // for spinner in beatmap.spinners.iter() {
        // }

        s.notes.sort_by(|a, b|a.time().partial_cmp(&b.time()).unwrap());
        s.end_time = s.notes.iter().last().unwrap().time();

        s
    }

    fn handle_replay_frame(&mut self, frame:ReplayFrame, manager:&mut IngameManager) {
        let time = manager.time();
        if !manager.replaying {
            manager.replay.frames.push((time, frame.clone()));
        }
        // let key = match frame {
        //     ReplayFrame::Press(k) => k,
        //     ReplayFrame::Release(k) => k
        // };

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
        if self.timing_point_index + 1 < timing_points.len() && timing_points[self.timing_point_index + 1].time <= time {
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

        let od = beatmap.metadata.od;
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


    fn timing_bar_things(&self) -> (Vec<(f32,Color)>, (f32,Color)) {
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

