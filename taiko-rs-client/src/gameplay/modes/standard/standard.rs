use ayyeve_piston_ui::render::*;
use piston::{MouseButton, RenderArgs};

use crate::game::Settings;
use crate::{Vector2, window_size};
use crate::helpers::{curve::get_curve, key_counter::KeyCounter};
use crate::gameplay::modes::{FIELD_SIZE, scale_coords, standard::*};
use taiko_rs_common::types::{KeyPress, ReplayFrame, ScoreHit, PlayMode};
use crate::gameplay::{DURATION_HEIGHT, GameMode, Beatmap, IngameManager, map_difficulty, defs::NoteType};

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

    draw_points: Vec<(f32, Vector2, ScoreHit)>,
    mouse_pos: Vector2,


    key_counter: KeyCounter,

    /// original, mouse_start
    move_playfield: Option<(Vector2, Vector2)>
}
impl StandardGame {
    pub fn next_note(&mut self) {self.note_index += 1}
    
    // pub fn next_slider(&mut self) {}
    // pub fn next_spinner(&mut self) {}
}

impl GameMode for StandardGame {
    fn playmode(&self) -> PlayMode {PlayMode::Standard}
    fn end_time(&self) -> f32 {self.end_time}
    fn new(beatmap:&Beatmap) -> Self {
        let settings = Settings::get_mut().standard_settings.clone();

        let mut s = Self {
            notes: Vec::new(),
            mouse_pos:Vector2::zero(),

            note_index: 0,
            end_time: 0.0,

            hitwindow_100: 0.0,
            hitwindow_300: 0.0,
            hitwindow_miss: 0.0,
            draw_points: Vec::new(),

            move_playfield: None,

            key_counter: KeyCounter::new(
                vec![
                    settings.left_key,
                    settings.right_key
                ],
                Vector2::zero()
            )
        };

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
                    spinner.clone()
                )))
            }

        }

        // s.notes.sort_by(|a, b|a.time().partial_cmp(&b.time()).unwrap());
        s.end_time = s.notes[s.notes.len() - 1].end_time(100.0);
        s
    }

    fn handle_replay_frame(&mut self, frame:ReplayFrame, time:f32, manager:&mut IngameManager) {
        if !manager.replaying {
            manager.replay.frames.push((time, frame.clone()));
        }
        if self.note_index >= self.notes.len() {return}

        match frame {
            ReplayFrame::Press(KeyPress::Left)
            | ReplayFrame::Press(KeyPress::Right) => {
                let pts = self.notes[self.note_index].get_points(time, (self.hitwindow_miss, self.hitwindow_100, self.hitwindow_300));
                let note_time = self.notes[self.note_index].time();
                self.draw_points.push((time, self.notes[self.note_index].point_draw_pos(), pts.clone()));
                match pts {
                    ScoreHit::Miss => {
                        println!("miss (press)");
                        manager.score.hit_miss(time, note_time);
                        manager.hitbar_timings.push((time, time - note_time));
                        if self.notes[self.note_index].note_type() == NoteType::Note {
                            self.next_note()
                        }
                    }
                    ScoreHit::X100 => {
                        {
                            let hitsound = self.notes[self.note_index].get_hitsound();
                            let hitsamples = self.notes[self.note_index].get_hitsamples().clone();
                            manager.play_note_sound(note_time, hitsound, hitsamples);
                        }

                        // Audio::play_preloaded("don");
                        manager.score.hit100(time, note_time);
                        manager.hitbar_timings.push((time, time - note_time));
                        if self.notes[self.note_index].note_type() == NoteType::Note {
                            self.next_note()
                        }
                    }
                    ScoreHit::X300 => {
                        {
                            let hitsound = self.notes[self.note_index].get_hitsound();
                            let hitsamples = self.notes[self.note_index].get_hitsamples().clone();
                            manager.play_note_sound(note_time, hitsound, hitsamples);
                        }

                        // Audio::play_preloaded("kat");
                        manager.score.hit300(time, note_time);
                        manager.hitbar_timings.push((time, time - note_time));
                        if self.notes[self.note_index].note_type() == NoteType::Note {
                            self.next_note()
                        }
                    }

                    ScoreHit::Other(_, _) => {}
                    ScoreHit::None => {},
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
                    self.draw_points.push((time, self.notes[self.note_index].point_draw_pos(), pts));
                    match pts {
                        ScoreHit::Miss => {
                            println!("miss (press)");
                            manager.score.hit_miss(time, note_time);
                            manager.hitbar_timings.push((time, time - note_time));
                            self.next_note()
                        },
                        ScoreHit::X100 => {
                            {
                                let hitsound = self.notes[self.note_index].get_hitsound();
                                let hitsamples = self.notes[self.note_index].get_hitsamples().clone();
                                manager.play_note_sound(note_time, hitsound, hitsamples);
                            }

                            manager.score.hit100(time, note_time);
                            manager.hitbar_timings.push((time, time - note_time));
                            self.next_note();
                        },
                        ScoreHit::X300 => {
                            {
                                let hitsound = self.notes[self.note_index].get_hitsound();
                                let hitsamples = self.notes[self.note_index].get_hitsamples().clone();
                                manager.play_note_sound(note_time, hitsound, hitsamples);
                            }

                            manager.score.hit300(time, note_time);
                            manager.hitbar_timings.push((time, time - note_time));
                            self.next_note();
                        },
                        ScoreHit::Other(_, _) => {}
                        ScoreHit::None => {}
                    }
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
        for note in self.notes.iter_mut() {
            note.update(time);

            // play queued sounds
            for (time, hitsound, mut samples, override_name) in note.get_sound_queue() {
                samples.filename = override_name;

                manager.play_note_sound(time, hitsound, samples);
            }
        }

        
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
        let end_time = self.notes[self.note_index].end_time(self.hitwindow_miss);
        let ntype = self.notes[self.note_index].note_type();
        let flag = match ntype {
            NoteType::Note => end_time < time,
            NoteType::Slider 
            | NoteType::Spinner => end_time <= time,
            _ => false,
        };

        if flag {
            match ntype {
                NoteType::Note => {
                    // need to set these manually instead of score.hit_miss,
                    // since we dont want to add anything to the hit error list
                    let note_time = self.notes[self.note_index].time();
                    manager.score.hit_miss(time, note_time);
                    // println!("note miss (time: {}, {}, diff: {}, od: {})", time, note_time, time - note_time, manager.beatmap.metadata.od);
                    self.draw_points.push((time, self.notes[self.note_index].point_draw_pos(), ScoreHit::Miss));
                }
                NoteType::Slider => {
                    let note_time = self.notes[self.note_index].end_time(0.0);
                    // check slider release points
                    // -1.0 for miss hitwindow to indidate it was held to the end (ie, no hitwindow to check)
                    let pts = self.notes[self.note_index].get_points(time, (-1.0, self.hitwindow_100, self.hitwindow_300));
                    self.draw_points.push((time, self.notes[self.note_index].point_draw_pos(), pts));
                    match pts {
                        ScoreHit::None | ScoreHit::Miss => {
                            manager.score.hit_miss(time, note_time);
                            manager.hitbar_timings.push((time, time - note_time));
                        },
                        ScoreHit::X100 => {
                            {
                                println!("sound on slider end: {} - {} = {}", time, note_time, time - note_time);
                                let hitsound = self.notes[self.note_index].get_hitsound();
                                let hitsamples = self.notes[self.note_index].get_hitsamples().clone();
                                manager.play_note_sound(note_time, hitsound, hitsamples);
                            }

                            manager.score.hit100(time, note_time);
                            manager.hitbar_timings.push((time, time - note_time));
                        },
                        ScoreHit::X300 => {
                            {
                                let hitsound = self.notes[self.note_index].get_hitsound();
                                let hitsamples = self.notes[self.note_index].get_hitsamples().clone();
                                manager.play_note_sound(note_time, hitsound, hitsamples);
                            }

                            manager.score.hit300(time, note_time);
                            manager.hitbar_timings.push((time, time - note_time));
                        },
                        ScoreHit::Other(_, _) => {}
                    }
                }
                NoteType::Spinner => {}

                _ => {},
            }
            self.next_note();
        }

    }
    fn draw(&mut self, args:RenderArgs, manager:&mut IngameManager, list:&mut Vec<Box<dyn Renderable>>) {
        // draw the playfield

        let p1 = scale_coords(Vector2::zero());
        let p2 = scale_coords(FIELD_SIZE);
        let playfield = Rectangle::new(
            [0.2, 0.2, 0.2, 0.5].into(),
            f64::MAX-4.0,
            p1,
            p2 - p1,
            // Vector2::new(0.0, HIT_POSITION.y - (PLAYFIELD_RADIUS + 2.0)),
            // Vector2::new(args.window_size[0], (PLAYFIELD_RADIUS+2.0) * 2.0),
            if manager.current_timing_point().kiai {
                Some(Border::new(Color::YELLOW, 2.0))
            } else {None}
        );
        list.push(Box::new(playfield));

        // draw key counter
        self.key_counter.draw(args, list);


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
        if key == piston::Key::LCtrl {
            let old = Settings::get_mut().standard_settings.get_playfield();
            self.move_playfield = Some((old.1, self.mouse_pos));
            return;
        }

        self.key_counter.key_press(key);
        let time = manager.time();

        let settings = Settings::get().standard_settings;
        if key == settings.left_key {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Left), time, manager);
        }
        if key == settings.right_key {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Right), time, manager);
        }
    }
    fn key_up(&mut self, key:piston::Key, manager:&mut IngameManager) {
        if key == piston::Key::LCtrl {
            self.move_playfield = None;
            return;
        }
        let time = manager.time();

        let settings = Settings::get().standard_settings;
        if key == settings.left_key {
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::Left), time, manager);
        }
        if key == settings.right_key {
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::Right), time, manager);
        }
    }
    
    fn mouse_move(&mut self, pos:Vector2, manager:&mut IngameManager) {
        self.mouse_pos = pos;
        let time = manager.time();

        if let Some((original, mouse_start)) = self.move_playfield {

            {
                let settings = &mut Settings::get_mut().standard_settings;
                let change = original + (pos - mouse_start);

                settings.playfield_x_offset = change.x;
                settings.playfield_y_offset = change.y;
            }

            // update playfield for notes
            for note in self.notes.iter_mut() {
                note.playfield_changed();
            }

            return;
        }

        self.handle_replay_frame(ReplayFrame::MousePos(pos.x as f32, pos.y as f32), time, manager);
    }
    fn mouse_down(&mut self, btn:piston::MouseButton, manager:&mut IngameManager) {
        {
            let settings = &Settings::get_mut().standard_settings;
            if settings.ignore_mouse_buttons {return}
        }

        let time = manager.time();
        if btn == MouseButton::Left {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Left), time, manager);
        }
        if btn == MouseButton::Right {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Right), time, manager);
        }
    }
    fn mouse_up(&mut self, btn:piston::MouseButton, manager:&mut IngameManager) {
        {
            let settings = &Settings::get_mut().standard_settings;
            if settings.ignore_mouse_buttons {return}
        }

        let time = manager.time();
        if btn == MouseButton::Left {
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::Left), time, manager);
        }
        if btn == MouseButton::Right {
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::Right), time, manager);
        }
    }

    fn mouse_scroll(&mut self, delta:f64, _manager:&mut IngameManager) {
        if self.move_playfield.is_some() {
            {
                let settings = &mut Settings::get_mut().standard_settings;
                settings.playfield_scale += delta / 40.0;
            }

            // update playfield for notes
            for note in self.notes.iter_mut() {
                note.playfield_changed();
            }
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
            Vector2::new(0.0, window_size().y - (size.y + DURATION_HEIGHT)),
            size
        )
    }
}

