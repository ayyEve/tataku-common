use piston::RenderArgs;
use ayyeve_piston_ui::render::*;

use super::*;
use crate::game::{Audio, Settings};
use crate::{window_size, Vector2, helpers::curve::get_curve};
use taiko_rs_common::types::{KeyPress, ReplayFrame, ScoreHit, PlayMode};
use crate::gameplay::{Beatmap, GameMode, IngameManager, map_difficulty, defs::*, modes::FIELD_SIZE};

// const SV_FACTOR:f64 = 700.0; // bc sv is bonked, divide it by this amount

pub const FRUIT_RADIUS_BASE:f64 = 20.0;
pub const DROPLET_RADIUS_BASE:f64 = 10.0;
pub const CATCHER_WIDTH_BASE:f64 = 106.75;
pub const CATCHER_BASE_SPEED:f64 = 1.0;

pub fn hit_y() -> f64 {
    window_size().y - 100.0
}
// pub const HIT_Y:f64 = window_size().y - 100.0

fn x_offset() -> f64 {
    (window_size().x - FIELD_SIZE.x) / 2.0
}
// const X_OFFSET:f64 = (window_size().x - FIELD_SIZE.x) / 2.0;


pub struct CatchGame {
    // lists
    pub notes: Vec<Box<dyn CatchHitObject>>,
    // list indices
    note_index: usize,
    timing_point_index: usize,

    // hit timing bar stuff
    hitwindow: f32,

    /// when does the map end
    end_time: f32,

    /// when was the last update
    last_update: f32,
    catcher: Catcher,
}
impl CatchGame {
    pub fn next_note(&mut self) {self.note_index += 1}
}
impl GameMode for CatchGame {
    fn playmode(&self) -> PlayMode {PlayMode::Catch}
    fn end_time(&self) -> f32 {self.end_time}
    fn new(beatmap:&Beatmap) -> Self {
        let mut s = Self {
            notes: Vec::new(),
            note_index: 0,

            timing_point_index: 0,
            end_time: 0.0,
            last_update: 0.0,

            hitwindow: 0.0,
            catcher: Catcher::new(&beatmap),
        };

        let x_offset = x_offset(); // (window_size().x - FIELD_SIZE.x) / 2.0;

        // add notes
        for note in beatmap.notes.iter() {
            //TODO!
            s.notes.push(Box::new(CatchFruit::new(
                note.time,
                1.0,
                FRUIT_RADIUS_BASE, 
                note.pos.x + x_offset
            )));
        }
        for slider in beatmap.sliders.iter() {
            let SliderDef {time, slides, length, ..} = slider.to_owned();

            let curve = get_curve(&slider, &beatmap);

            let l = (length * 1.4) * slides as f32;
            let v2 = 100.0 * (beatmap.metadata.slider_multiplier * 1.4);
            let bl = beatmap.beat_length_at(time, true);
            let end_time = time + (l / v2 * bl);
            // let end_time = curve.end_time;
            
            let bl = beatmap.beat_length_at(time, beatmap.metadata.beatmap_version < 8);
            let skip_period = (bl / beatmap.metadata.slider_tick_rate).min((end_time - time) / slides as f32);

            let mut j = time;

            // // load sounds
            // // let sound_list_raw = if let Some(list) = split.next() {list.split("|")} else {"".split("")};

            // // when loading, if unified just have it as sound_types with 1 index
            // // let mut sound_types:Vec<(HitType, bool)> = Vec::new();

            // // for i in sound_list_raw {
            // //     if let Ok(hitsound) = i.parse::<u32>() {
            // //         let hit_type = if (hitsound & (2 | 8)) > 0 {super::HitType::Kat} else {super::HitType::Don};
            // //         let finisher = (hitsound & 4) > 0;
            // //         sound_types.push((hit_type, finisher));
            // //     }
            // // }
            
            // // let unified_sound_addition = sound_types.len() == 0;
            // // if unified_sound_addition {
            // //     sound_types.push((HitType::Don, false));
            // // }
            // // println!("{:?}", points);


            let mut counter = 0;
            loop {
                // let sound_type = sound_types[i];
                if counter % 4 == 0 {
                    s.notes.push(Box::new(CatchFruit::new(
                        j,
                        1.0,//beatmap.slider_velocity_at(j as u64),
                        FRUIT_RADIUS_BASE,
                        curve.position_at_time(j).x + x_offset
                    )));
                } else {
                    s.notes.push(Box::new(CatchDroplet::new(
                        j,
                        1.0,//beatmap.slider_velocity_at(j as u64),
                        DROPLET_RADIUS_BASE,
                        curve.position_at_time(j).x + x_offset
                    )));
                }

                // if !unified_sound_addition {i = (i + 1) % sound_types.len()}
                j += skip_period;
                counter += 1;
                if !(j < end_time + skip_period / 8.0) {break}
            }
        }
        for spinner in beatmap.spinners.iter() {
            let SpinnerDef {time, end_time, ..} = spinner;
            let length = end_time - time;
            for i in (0..length as i32).step_by(50) {
                s.notes.push(Box::new(CatchBanana::new(
                    time + i as f32,
                    1.0,
                    5.0,
                    i as f64 % FIELD_SIZE.x + x_offset
                )))
            }
        }

        s.notes.sort_by(|a, b|a.time().partial_cmp(&b.time()).unwrap());




        // // set dashes
        // // from lazer CatchBeatmapProcessor:214
        // let half_catcher = s.catcher.width / 2.0;
        // let mut last_direction = 0;
        // let mut last_excess = half_catcher;

        // let mut i = 0;
        // let notes = &mut s.notes;
        // 'dash_loop: while i < notes.len() - 1 {
        //     while notes[i].note_type() == NoteType::Spinner {
        //         i += 1;
        //         if i >= notes.len() - 1 {
        //             break 'dash_loop;
        //         }
        //     }
        //     let current = i;
        //     let mut next = i + 1;
        //     while notes[next].note_type() == NoteType::Spinner {
        //         next += 1;
        //         if next >= notes.len() - 1 {
        //             break 'dash_loop;
        //         }
        //     }

        //     // reset dash values
        //     notes[current].reset_dash();
        //     let this_direction = if notes[next].x() > notes[current].x() {-1} else {1};
        //     let time_to_next = notes[next].time() as f64 - notes[current].time() as f64 - 1000.0 / 60.0 / 4.0;
        //     let distance_to_next = 
        //         (notes[next].x() - notes[current].x()) 
        //         - (if last_direction == this_direction {last_excess} else {half_catcher});
            
        //     let distance_to_hyper = time_to_next * CATCHER_BASE_SPEED - distance_to_next;

        //     // if distance_to_hyper < 0.0 {
        //     //     notes[current].set_dash(&notes[next]);
        //     // } else {
        //     //     notes[current].set_hyper_distance()
        //     // }

        //     i += 1;
        // };


        s.end_time = s.notes.iter().last().unwrap().time();
        s
    }

    fn update(&mut self, manager:&mut IngameManager, time: f32) {
        self.catcher.update(time as f64);

        // update notes
        for note in self.notes.iter_mut() {note.update(time)}

        // if theres no more notes to hit, show score screen
        if self.note_index >= self.notes.len() {
            manager.completed = true;
            return;
        }

        // check if we missed the current note
        let note_time = self.notes[self.note_index].time();
        if note_time < time {
            if self.notes[self.note_index].causes_miss() {
                let s = &mut manager.score;
                s.xmiss += 1;
                s.combo = 0;
            }
            self.next_note();
        } else if ((note_time - time).abs()) < self.hitwindow {
            let note = self.notes.get_mut(self.note_index).unwrap();

            if self.catcher.catches(note) {
                match note.get_points() {
                    ScoreHit::X300 => {
                        manager.score.hit300(time, note_time);
                        manager.hitbar_timings.push((time, 3.0));
                        self.next_note();
                    }
                    ScoreHit::X100 => {
                        manager.score.hit100(time, note_time);
                        manager.hitbar_timings.push((time, 2.0));
                        self.next_note();
                    }
                    ScoreHit::Other(score, _consume) => { // spinner drop
                        manager.score.score += score as u64;
                        manager.hitbar_timings.push((time, 1.0));
                        self.next_note();
                    }
                    _ => {}
                }

                Audio::play_preloaded("don");
            } else {
                if note.causes_miss() {
                    let s = &mut manager.score;
                    s.xmiss += 1;
                    s.combo = 0;
                }
            }
        }

        let timing_points = &manager.beatmap.timing_points;
        // check timing point
        if self.timing_point_index + 1 < timing_points.len() && timing_points[self.timing_point_index + 1].time <= time {
            self.timing_point_index += 1;
        }

        self.last_update = time;
    }
    fn draw(&mut self, args:RenderArgs, manager:&mut IngameManager, list:&mut Vec<Box<dyn Renderable>>) {
        // draw the playfield
        let playfield = Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            f64::MAX-4.0,
            Vector2::new(x_offset(), 0.0),
            Vector2::new(FIELD_SIZE.x, args.window_size[1]),
            if manager.beatmap.timing_points[self.timing_point_index].kiai {
                Some(Border::new(Color::YELLOW, 2.0))
            } else {None}
        );
        list.push(Box::new(playfield));
        self.catcher.draw(list);

        // for curve in self.curves.iter() {
        //     for line in curve.path.iter() {
        //         list.push(Box::new(ayyeve_piston_ui::render::Line::new(
        //             line.p1,
        //             line.p2,
        //             5.0,
        //             -999.0,
        //             Color::GREEN
        //         )))
        //     }
        // }

        // draw notes
        for note in self.notes.iter_mut() {note.draw(args, list)}
    }


    fn handle_replay_frame(&mut self, frame:ReplayFrame, manager:&mut IngameManager) {
        let time = manager.time();
        if !manager.replaying {
            manager.replay.frames.push((time, frame.clone()));
        }
        
        match frame {
            ReplayFrame::Press(k) => {
                match k {
                    KeyPress::Left => self.catcher.left_held = true,
                    KeyPress::Right => self.catcher.right_held = true,
                    KeyPress::Dash => self.catcher.dash_held = true,
                    _ => {}
                }
            }
            ReplayFrame::Release(k) => {
                match k {
                    KeyPress::Left => self.catcher.left_held = false,
                    KeyPress::Right => self.catcher.right_held = false,
                    KeyPress::Dash => self.catcher.dash_held = false,
                    _ => {}
                }
            }
            
            _ => {}
        };
    }
    fn key_down(&mut self, key:piston::Key, manager:&mut IngameManager) {
        let settings = Settings::get().catch_settings;

        if !manager.replaying {
            if key == settings.left_key {
                self.handle_replay_frame(ReplayFrame::Press(KeyPress::Left), manager);
            }
            if key == settings.right_key {
                self.handle_replay_frame(ReplayFrame::Press(KeyPress::Right), manager);
            }
            if key == settings.dash_key { 
                self.handle_replay_frame(ReplayFrame::Press(KeyPress::Dash), manager);
            }
        }
    }
    fn key_up(&mut self, key:piston::Key, manager:&mut IngameManager) {
        let settings = Settings::get().catch_settings;

        if !manager.replaying {
            if key == settings.left_key {
                self.handle_replay_frame(ReplayFrame::Release(KeyPress::Left), manager);
            }
            if key == settings.right_key {
                self.handle_replay_frame(ReplayFrame::Release(KeyPress::Right), manager);
            }
            if key == settings.dash_key { 
                self.handle_replay_frame(ReplayFrame::Release(KeyPress::Dash), manager);
            }
        }
    }

    fn reset(&mut self, beatmap:&Beatmap) {
        for note in self.notes.as_mut_slice() {
            note.reset();
        }
        
        self.note_index = 0;
        self.timing_point_index = 0;

        let od = beatmap.metadata.od;
        self.hitwindow = map_difficulty(od, 50.0, 35.0, 20.0);
    }

    fn skip_intro(&mut self, manager: &mut IngameManager) {
        if self.note_index > 0 {return}

        let y_needed = 0.0;
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
        (vec![], (0.0, Color::RED))
    }
    fn combo_bounds(&self) -> Rectangle {
        Rectangle::bounds_only(
            Vector2::new(0.0, window_size().y * (1.0/3.0)),
            Vector2::new(window_size().x, 30.0)
        )
    }
}

struct Catcher {
    width: f64,
    pos: Vector2,

    pub left_held: bool,
    pub right_held: bool,
    pub dash_held: bool,

    last_update: f64,
    move_speed: f64
}
impl Catcher {
    fn new(_beatmap:&Beatmap) -> Self {
        let width = 100.0;
        Self {
            width,
            move_speed: 0.5, // should calc this somehow
            pos: Vector2::new((window_size().x - width) / 2.0, hit_y()),
            left_held: false,
            right_held: false,
            dash_held: false,
            last_update: 0.0
        }
    }

    fn update(&mut self, time:f64) {
        if self.last_update == 0.0 {return self.last_update = time}
        let delta = time - self.last_update;

        if self.left_held {
            self.pos.x -= self.move_speed() * delta;
        }
        if self.right_held {
            self.pos.x += self.move_speed() * delta;
        }

        let x_offset = x_offset();
        // check bounds
        if self.pos.x < x_offset {
            self.pos.x = x_offset;
        }
        if self.pos.x + self.width > x_offset + FIELD_SIZE.x {
            self.pos.x = x_offset + FIELD_SIZE.x - self.width;
        }
        self.last_update = time;
    }
    fn move_speed(&self) -> f64 {
        if self.dash_held {
            self.move_speed * 1.5 //TODO!!
        } else {
            self.move_speed
        }
    }
    fn draw(&mut self, list:&mut Vec<Box<dyn Renderable>>) {
        list.push(Box::new(Rectangle::new(
            Color::BLUE,
            -100.0,
            self.pos,
            Vector2::new(self.width, 10.0),
            None
        )))
    }

    fn catches(&self, note: &Box<dyn CatchHitObject>) -> bool {
        let note_x = note.x();
        let note_width = note.radius() * 2.0;

        note_x + note_width > self.pos.x && note_x < self.pos.x + self.width
    }
}