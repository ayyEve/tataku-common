use ayyeve_piston_ui::render::*;
use piston::{MouseButton, RenderArgs};

use super::*;
use crate::helpers::slider::get_curve;
use crate::{window_size, Vector2, game::Settings};
use taiko_rs_common::types::{KeyPress, ReplayFrame, ScoreHit, PlayMode};
use crate::gameplay::{GameMode, Beatmap, IngameManager, map_difficulty, modes::FIELD_SIZE, defs::NoteType};

const POINTS_DRAW_TIME:f32 = 100.0;
const POINTS_DRAW_FADE_TIME:f32 = 40.0;

pub struct StandardGame {
    // lists
    pub notes: Vec<Box<dyn StandardHitObject>>,
    
    /// where to start checking notes from
    note_index: usize,

    // hit timing bar stuff
    hitwindow_300: f32,
    hitwindow_100: f32,
    hitwindow_miss: f32,
    end_time: f32,

    draw_points: Vec<(f32, Vector2, ScoreHit)>
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

            end_time: 0.0,

            hitwindow_100: 0.0,
            hitwindow_300: 0.0,
            hitwindow_miss: 0.0,
            draw_points: Vec::new()
        };

        // let ar = beatmap.metadata.
        let ar = beatmap.metadata.ar;
        let cs = beatmap.metadata.cs;

        // join notes and sliders into a single array
        // needed because of combo counts
        let mut all_items = Vec::new();
        for note in beatmap.notes.iter() {
            all_items.push((Some(note), None, None))
        }
        for slider in beatmap.sliders.iter() {
            all_items.push((None, Some(slider), None))
        }
        for spinner in beatmap.spinners.iter() {
            all_items.push((None, None, Some(spinner)))
        }
        // sort
        all_items.sort_by(|a, b| {
            let a_time = match a {
                (Some(note), None, None) => note.time,
                (None, Some(slider), None) => slider.time,
                (None, None, Some(spinner)) => spinner.time,
                _ => 0.0
            };
            let b_time = match b {
                (Some(note), None, None) => note.time,
                (None, Some(slider), None) => slider.time,
                (None, None, Some(spinner)) => spinner.time,
                _ => 0.0
            };

            a_time.partial_cmp(&b_time).unwrap()
        });


        // add notes
        let mut combo_num = 0;
        let mut combo_change = 0;
        let combo_colors = [
            Color::new(0.8, 0.0, 0.0, 1.0),
            Color::new(0.8, 0.8, 0.0, 1.0),
            Color::new(0.0, 0.8, 0.8, 1.0),
            Color::new(0.0, 0.0, 0.8, 1.0)
        ];

        for (note, slider, spinner) in all_items {
            // check for new combo
            if let Some(note) = note {if note.new_combo {combo_num = 0}}
            if let Some(slider) = slider {if slider.new_combo {combo_num = 0}}
            if let Some(spinner) = spinner {if spinner.new_combo {combo_num = 0}}
            // if new combo, increment new combo counter
            if combo_num == 0 {combo_change += 1}
            // get color
            let color = combo_colors[(combo_change - 1) % combo_colors.len()];
            // update combo number
            combo_num += 1;


            if let Some(note) = note {
                s.notes.push(Box::new(StandardNote::new(
                    note.clone(),
                    ar,
                    cs,
                    color,
                    combo_num as u16
                )));
            }
            if let Some(slider) = slider {
                let curve = get_curve(slider, &beatmap);
                s.notes.push(Box::new(StandardSlider::new(
                    slider.clone(),
                    curve,
                    ar,
                    cs,
                    color,
                    combo_num as u16
                )))
            }
            if let Some(spinner) = spinner {
                s.notes.push(Box::new(StandardSpinner::new(
                    spinner.time,
                    spinner.end_time
                )))
            }

        }

        // s.notes.sort_by(|a, b|a.time().partial_cmp(&b.time()).unwrap());
        s.end_time = s.notes[s.notes.len() - 1].end_time(100.0);
        s
    }

    fn handle_replay_frame(&mut self, frame:ReplayFrame, manager:&mut IngameManager) {
        let time = manager.time();
        if !manager.replaying {
            manager.replay.frames.push((time, frame.clone()));
        }
        if self.note_index >= self.notes.len() {return}

        match frame {
            ReplayFrame::Press(KeyPress::Left)
            | ReplayFrame::Press(KeyPress::Right) => {
                let pts = self.notes[self.note_index].get_points(time, (self.hitwindow_miss, self.hitwindow_100, self.hitwindow_300));
                let note_time = self.notes[self.note_index].time();
                match pts {
                    ScoreHit::Miss => {
                        println!("miss (press)");
                        manager.score.hit_miss(time, note_time);
                    },
                    ScoreHit::X100 => manager.score.hit100(time, note_time),
                    ScoreHit::X300 => manager.score.hit300(time, note_time),
                    ScoreHit::Other(_, _) => {}
                    ScoreHit::None => {},
                }

                self.draw_points.push((time, self.notes[self.note_index].point_draw_pos(), pts.clone()));

                // dont do the next note for sliders and spinners
                if self.notes[self.note_index].note_type() == NoteType::Note {
                    // check miss
                    match pts {
                        ScoreHit::None => {},
                        _ => self.next_note(),
                    }
                }

                
                // self.notes[self.note_index].press(time);
                for note in self.notes.iter_mut() {
                    note.press(time)
                }
            }
            ReplayFrame::Release(KeyPress::Left) 
            | ReplayFrame::Release(KeyPress::Right) => {
                if self.notes[self.note_index].note_type() == NoteType::Slider {
                    let pts = self.notes[self.note_index].get_points(time, (self.hitwindow_miss, self.hitwindow_100, self.hitwindow_300));
                    let note_time = self.notes[self.note_index].time();
                    match pts {
                        ScoreHit::Miss => {
                            println!("slider miss (release)");
                            manager.score.combo = 0;
                            // manager.score.hit_miss(time, note_time);
                        },
                        ScoreHit::X100 => manager.score.hit100(time, note_time),
                        ScoreHit::X300 => manager.score.hit300(time, note_time),
                        ScoreHit::Other(_, _) => {}
                        ScoreHit::None => {},
                    }
                    self.draw_points.push((time, self.notes[self.note_index].point_draw_pos(), pts));
                }

                // self.notes[self.note_index].release(time);
                for note in self.notes.iter_mut() {
                    note.release(time)
                }
            }
            ReplayFrame::MousePos(x, y) => {
                for note in self.notes.iter_mut() {
                    note.mouse_move(Vector2::new(x as f64, y as f64));
                }
                // self.notes[self.note_index]
            }
            _ => {}
        }
    }


    fn update(&mut self, manager:&mut IngameManager, time:f32) {
        // update notes
        for note in self.notes.iter_mut() {note.update(time)}
        
        // remove old draw points
        self.draw_points.retain(|a| time < a.0 + POINTS_DRAW_TIME);

        // if theres no more notes to hit, show score screen
        if self.note_index >= self.notes.len() {
            manager.completed = true;
            return;
        }

        // since some std maps are non-linear (2b),
        // we need to check all notes up until a certain criteria 
        // TODO! figure out this criteria

        // check if we missed the current note
        if self.notes[self.note_index].end_time(self.hitwindow_miss) < time {
            if self.notes[self.note_index].note_type() == NoteType::Note {
                // need to set these manually instead of score.hit_miss,
                // since we dont want to add anything to the hit error list
                let s = &mut manager.score;
                s.xmiss += 1;
                s.combo = 0;
                println!("note miss (time)");
                self.draw_points.push((time, self.notes[self.note_index].point_draw_pos(), ScoreHit::Miss));
            } else {
                // check slider points
                let pts = self.notes[self.note_index].get_points(time, (self.hitwindow_miss, self.hitwindow_100, self.hitwindow_300));
                match pts {
                    ScoreHit::None => {}
                    ScoreHit::Miss => {}
                    ScoreHit::X100 => {}
                    ScoreHit::X300 => {}
                    ScoreHit::Other(_, _) => {}
                }
                self.draw_points.push((time, self.notes[self.note_index].point_draw_pos(), pts));
            }
            self.next_note();
        }
    }
    fn draw(&mut self, args:RenderArgs, manager:&mut IngameManager, list:&mut Vec<Box<dyn Renderable>>) {
        // draw the playfield
        let playfield = Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            f64::MAX-4.0,
            pos_offset(),
            FIELD_SIZE,
            // Vector2::new(0.0, HIT_POSITION.y - (PLAYFIELD_RADIUS + 2.0)),
            // Vector2::new(args.window_size[0], (PLAYFIELD_RADIUS+2.0) * 2.0),
            if manager.current_timing_point().kiai {
                Some(Border::new(Color::YELLOW, 2.0))
            } else {None}
        );
        list.push(Box::new(playfield));


        let time = manager.time();
        for (p_time, pos, pts) in self.draw_points.iter() {
            let mut color;
            match pts {
                ScoreHit::None => continue,
                ScoreHit::Miss => color = Color::RED,
                ScoreHit::X100 => color = Color::GREEN,
                ScoreHit::X300 => color = Color::new(0.0, 0.7647, 1.0, 1.0),
                ScoreHit::Other(_, _) => continue,
            }
            
            let diff = time - p_time;
            if diff > POINTS_DRAW_TIME - POINTS_DRAW_FADE_TIME {
                color.a = 1.0 - (diff-POINTS_DRAW_TIME) / POINTS_DRAW_FADE_TIME;
            }

            list.push(Box::new(Circle::new(
                color,
                -99_999.9,
                *pos,
                20.0
            )))
        }


        // draw notes
        for note in self.notes.iter_mut() {note.draw(args, list)}
    }


    fn key_down(&mut self, key:piston::Key, manager:&mut IngameManager) {
        let settings = Settings::get().standard_settings;
        if key == settings.left_key {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Left), manager);
        }
        if key == settings.right_key {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Right), manager);
        }
    }
    fn key_up(&mut self, key:piston::Key, manager:&mut IngameManager) {
        let settings = Settings::get().standard_settings;
        if key == settings.left_key {
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::Left), manager);
        }
        if key == settings.right_key {
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::Right), manager);
        }
    }
    fn mouse_move(&mut self, pos:Vector2, manager:&mut IngameManager) {
        self.handle_replay_frame(ReplayFrame::MousePos(pos.x as f32, pos.y as f32), manager);
    }

    fn mouse_down(&mut self, btn:piston::MouseButton, manager:&mut IngameManager) {
        if btn == MouseButton::Left {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Left), manager);
        }
        if btn == MouseButton::Right {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Right), manager);
        }
    }
    fn mouse_up(&mut self, btn:piston::MouseButton, manager:&mut IngameManager) {
        if btn == MouseButton::Left {
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::Left), manager);
        }
        if btn == MouseButton::Right {
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::Right), manager);
        }
    }

    fn reset(&mut self, beatmap:&Beatmap) {
        self.note_index = 0;
        
        for note in self.notes.as_mut_slice() {
            note.reset();
        }
        
        let od = beatmap.metadata.od;
        // setup hitwindows
        self.hitwindow_miss = map_difficulty(od, 135.0, 95.0, 70.0);
        self.hitwindow_100 = map_difficulty(od, 120.0, 80.0, 50.0);
        self.hitwindow_300 = map_difficulty(od, 50.0, 35.0, 20.0);

        self.draw_points.clear();
    }



    fn skip_intro(&mut self, manager: &mut IngameManager) {
        if self.note_index > 0 || self.notes.len() == 0 {return}

        let time = self.notes[0].time() - self.notes[0].get_preempt();
        if time < manager.time() {return}

        manager.song.upgrade().unwrap().set_position(time);
    }


    fn timing_bar_things(&self) -> (Vec<(f32,Color)>, (f32,Color)) {
        (vec![
            (self.hitwindow_100, [0.3411, 0.8901, 0.0745, 1.0].into()),
            (self.hitwindow_300, [0.0, 0.7647, 1.0, 1.0].into()),
        ], (self.hitwindow_miss, [0.8549, 0.6823, 0.2745, 1.0].into()))
    }

    fn combo_bounds(&self) -> Rectangle {
        let size = Vector2::new(100.0, 30.0);
        Rectangle::bounds_only(
            Vector2::new(0.0, window_size().y - size.y),
            size
        )
    }
}

