use ayyeve_piston_ui::render::*;
use piston::RenderArgs;

use super::*;
use crate::helpers::slider::get_curve;
use crate::{WINDOW_SIZE, Vector2, game::Settings};
use taiko_rs_common::types::{KeyPress, ReplayFrame, ScoreHit, PlayMode};
use crate::gameplay::{GameMode, Beatmap, IngameManager, map_difficulty, modes::FIELD_SIZE, defs::NoteType};

pub struct StandardGame {
    // lists
    pub notes: Vec<Box<dyn StandardHitObject>>,
    
    /// where to start checking notes from
    note_index: usize,

    // hit timing bar stuff
    hitwindow_300: f32,
    hitwindow_100: f32,
    hitwindow_miss: f32,

    end_time: f32
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
        };

        // let ar = beatmap.metadata.
        let ar = beatmap.metadata.ar;
        let cs = beatmap.metadata.cs;

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
                ar,
                cs,
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
        if self.notes.len() < self.note_index {return}

        match frame {
            ReplayFrame::Press(KeyPress::Left)
            | ReplayFrame::Press(KeyPress::Right) => {
                if self.notes[self.note_index].note_type() == NoteType::Note {
                    let pts = self.notes[self.note_index].get_points(time, (self.hitwindow_miss, self.hitwindow_100, self.hitwindow_300));
                    let note_time = self.notes[self.note_index].time();
                    match pts {
                        ScoreHit::Miss => {
                            println!("note miss (timing)");
                            manager.score.hit_miss(time, note_time);
                        },
                        ScoreHit::X100 => manager.score.hit100(time, note_time),
                        ScoreHit::X300 => manager.score.hit300(time, note_time),
                        ScoreHit::Other(_, _) => {}
                        ScoreHit::None => {},
                    }
                    self.next_note();
                } else {
                    self.notes[self.note_index].press(time);
                }
            }            
            ReplayFrame::Release(KeyPress::Left) 
            | ReplayFrame::Release(KeyPress::Right) => {
                self.notes[self.note_index].release(time);
                
            }
            ReplayFrame::MousePos(x, y) => {
                self.notes[self.note_index].mouse_move(Vector2::new(x as f64, y as f64));
            }
            _ => {}
        }
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

        // check if we missed the current note
        if self.notes[self.note_index].end_time(self.hitwindow_miss) < time {

            if self.notes[self.note_index].note_type() == NoteType::Note {
                // need to set these manually instead of score.hit_miss,
                // since we dont want to add anything to the hit error list
                let s = &mut manager.score;
                s.xmiss += 1;
                s.combo = 0;
                println!("note miss (time)");
            } else {
                // check slider points

            }
            self.next_note();
        }
        
    }
    fn draw(&mut self, args:RenderArgs, manager:&mut IngameManager, list:&mut Vec<Box<dyn Renderable>>) {

        // draw the playfield
        let playfield = Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            f64::MAX-4.0,
            POS_OFFSET,
            FIELD_SIZE,
            // Vector2::new(0.0, HIT_POSITION.y - (PLAYFIELD_RADIUS + 2.0)),
            // Vector2::new(args.window_size[0], (PLAYFIELD_RADIUS+2.0) * 2.0),
            if manager.current_timing_point().kiai {
                Some(Border::new(Color::YELLOW, 2.0))
            } else {None}
        );
        list.push(Box::new(playfield));


        // draw notes
        for note in self.notes.iter_mut() {note.draw(args, list)}
    }


    fn key_down(&mut self, key:piston::Key, manager:&mut IngameManager) {
        let settings = Settings::get().taiko_settings;
        if key == settings.left_kat {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Left), manager);
        }
        if key == settings.left_don {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Right), manager);
        }
    }
    fn key_up(&mut self, key:piston::Key, manager:&mut IngameManager) {
        let settings = Settings::get().taiko_settings;
        if key == settings.left_kat {
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::Left), manager);
        }
        if key == settings.left_don {
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::Right), manager);
        }
    }
    fn mouse_move(&mut self, pos:Vector2, manager:&mut IngameManager) {
        self.handle_replay_frame(ReplayFrame::MousePos(pos.x as f32, pos.y as f32), manager);
    }

    fn reset(&mut self, beatmap:Beatmap) {
        self.note_index = 0;
        
        for note in self.notes.as_mut_slice() {
            note.reset();
        }
        

        let od = beatmap.metadata.od;
        // setup hitwindows
        self.hitwindow_miss = map_difficulty(od, 135.0, 95.0, 70.0);
        self.hitwindow_100 = map_difficulty(od, 120.0, 80.0, 50.0);
        self.hitwindow_300 = map_difficulty(od, 50.0, 35.0, 20.0);

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

