use ayyeve_piston_ui::render::*;
use piston::{MouseButton, RenderArgs};

use crate::game::{Settings, StandardSettings};
use crate::gameplay::hitobject_defs::NoteDef;
use crate::{Vector2, window_size};
use crate::helpers::{curve::get_curve, key_counter::KeyCounter};
use crate::gameplay::modes::{FIELD_SIZE, ScalingHelper, standard::*};
use taiko_rs_common::types::{KeyPress, ReplayFrame, ScoreHit, PlayMode};
use crate::gameplay::{DURATION_HEIGHT, GameMode, Beatmap, IngameManager, map_difficulty, defs::NoteType};

const POINTS_DRAW_TIME:f32 = 100.0;
const POINTS_DRAW_FADE_TIME:f32 = 40.0;

pub struct StandardGame {
    // lists
    pub notes: Vec<Box<dyn StandardHitObject>>,

    // hit timing bar stuff
    hitwindow_300: f32,
    hitwindow_100: f32,
    hitwindow_50: f32,
    hitwindow_miss: f32,
    end_time: f32,

    draw_points: Vec<(f32, Vector2, ScoreHit)>,
    mouse_pos: Vector2,

    key_counter: KeyCounter,

    /// original, mouse_start
    move_playfield: Option<(Vector2, Vector2)>,

    /// how many keys are being held?
    hold_count: u8,

    /// scaling helper to help with scaling
    scaling_helper: ScalingHelper,
    /// needed for scaling recalc
    cs: f32,

    /// cached settings, saves on locking
    settings: StandardSettings
}
impl StandardGame {
    fn playfield_changed(&mut self) {
        let new_scale = ScalingHelper::new(self.cs, PlayMode::Standard);
        self.scaling_helper = new_scale;

        // update playfield for notes
        for note in self.notes.iter_mut() {
            note.playfield_changed(&new_scale);
        }
    }
}

impl GameMode for StandardGame {
    fn playmode(&self) -> PlayMode {PlayMode::Standard}
    fn end_time(&self) -> f32 {self.end_time}
    fn new(beatmap:&Beatmap) -> Self {
        let ar = beatmap.metadata.ar;
        let settings = Settings::get_mut().standard_settings.clone();
        let scaling_helper = ScalingHelper::new(beatmap.metadata.cs, PlayMode::Standard);

        let mut s = Self {
            notes: Vec::new(),
            mouse_pos:Vector2::zero(),

            hold_count: 0,
            // note_index: 0,
            end_time: 0.0,

            hitwindow_50: 0.0,
            hitwindow_100: 0.0,
            hitwindow_300: 0.0,
            hitwindow_miss: 0.0,
            draw_points: Vec::new(),

            move_playfield: None,
            scaling_helper,
            cs: beatmap.metadata.cs,

            key_counter: KeyCounter::new(
                vec![
                    settings.left_key,
                    settings.right_key
                ],
                Vector2::zero()
            ),

            settings,
        };


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
                    color,
                    combo_num as u16,
                    &scaling_helper
                )));
            }
            if let Some(slider) = slider {
                
                // invisible note
                if slider.curve_points.len() == 0 || slider.length == 0.0 {
                    let note = &NoteDef {
                        pos: slider.pos,
                        time: slider.time,
                        hitsound: slider.hitsound,
                        hitsamples: slider.hitsamples.clone(),
                        new_combo: slider.new_combo,
                        color_skip: slider.color_skip
                    };

                    s.notes.push(Box::new(StandardNote::new(
                        note.clone(),
                        ar,
                        Color::new(0.0, 0.0, 0.0, 1.0),
                        combo_num as u16,
                        &scaling_helper
                    )));
                } else {
                    let curve = get_curve(slider, &beatmap);
                    s.notes.push(Box::new(StandardSlider::new(
                        slider.clone(),
                        curve,
                        ar,
                        color,
                        combo_num as u16,
                        scaling_helper.clone()
                    )))
                }

            }
            if let Some(spinner) = spinner {
                s.notes.push(Box::new(StandardSpinner::new(
                    spinner.clone()
                )))
            }

        }

        // get the end pos
        for n in s.notes.iter() {
            s.end_time = s.end_time.max(n.end_time(0.0));
        }
        s.end_time += 1000.0;

        s
    }

    fn handle_replay_frame(&mut self, frame:ReplayFrame, time:f32, manager:&mut IngameManager) {
        if !manager.replaying {
            manager.replay.frames.push((time, frame.clone()));
        }

        match frame {
            ReplayFrame::Press(KeyPress::Left)
            | ReplayFrame::Press(KeyPress::Right) => {
                self.hold_count += 1;

                let pt_pos;
                let pts;
                {
                    let mut check_notes = Vec::new();
                    let w = self.hitwindow_miss;
                    for note in self.notes.iter_mut() {
                        // check if note is in hitwindow
                        if (time - note.time()).abs() <= w && !note.was_hit() {
                            check_notes.push(note);
                        }
                    }
                    if check_notes.len() == 0 {return} // no notes to check

                    check_notes.sort_by(|a, b| a.time().partial_cmp(&b.time()).unwrap());
                    let note = &mut check_notes[0];

                    pts = note.get_points(true, time, (self.hitwindow_miss, self.hitwindow_50, self.hitwindow_100, self.hitwindow_300));
                    let note_time = note.time();

                    match &pts {
                        ScoreHit::None | ScoreHit::Other(_,_) => {}
                        ScoreHit::Miss => {
                            println!("miss (press)");
                            manager.score.hit_miss(time, note_time);
                            manager.hitbar_timings.push((time, time - note_time));
                        }

                        pts => {
                            let hitsound = note.get_hitsound();
                            let hitsamples = note.get_hitsamples().clone();
                            manager.play_note_sound(note_time, hitsound, hitsamples);

                            match pts {
                                ScoreHit::X50 => manager.score.hit50(time, note_time),
                                ScoreHit::X100 => manager.score.hit100(time, note_time),
                                ScoreHit::X300 => manager.score.hit300(time, note_time),
                                _ => {}
                            }

                            manager.hitbar_timings.push((time, time - note_time));
                        }
                    }
                    
                    pt_pos = note.point_draw_pos();
                }

                self.draw_points.push((time, pt_pos, pts.clone()));

                // self.notes[self.note_index].press(time);
                for note in self.notes.iter_mut() {
                    note.press(time)
                }
            }
            ReplayFrame::Release(KeyPress::Left) 
            | ReplayFrame::Release(KeyPress::Right) => {
                if self.hold_count == 0 {return} // dont continue if no keys were being held (happens when leaving a menu)
                self.hold_count -= 1;

                let mut check_notes = Vec::new();
                let w = self.hitwindow_miss;
                for note in self.notes.iter_mut() {
                    // check if note is in hitwindow
                    if (time - note.time()).abs() <= w && !note.was_hit() {
                        check_notes.push(note);
                    }
                }
                if check_notes.len() == 0 {return} // no notes to check
                
                check_notes.sort_by(|a, b| a.time().partial_cmp(&b.time()).unwrap());
                let note = &mut check_notes[0];

                
                if note.note_type() == NoteType::Slider {
                    let pts = note.get_points(false, time, (self.hitwindow_miss, self.hitwindow_50, self.hitwindow_100, self.hitwindow_300));
                    let note_time = note.time();
                    self.draw_points.push((time, note.point_draw_pos(), pts));
                    match pts {
                        ScoreHit::Other(_, _) | ScoreHit::None => {}

                        ScoreHit::Miss => {
                            println!("slider miss (release)");
                            manager.score.combo = 0;
                        },

                        pts => {
                            match pts {
                                ScoreHit::X300 => manager.score.hit300(time, note_time),
                                ScoreHit::X100 => manager.score.hit100(time, note_time),
                                ScoreHit::X50  => manager.score.hit50 (time, note_time),
                                _ => {}
                            }

                            // play hitsound
                            let hitsound = note.get_hitsound();
                            let hitsamples = note.get_hitsamples().clone();
                            manager.play_note_sound(note_time, hitsound, hitsamples);
                            
                            // add to hit timing bar
                            // manager.hitbar_timings.push((time, time - note_time));
                        }
                    }
                }

                // if this is the last key to be released
                if self.hold_count == 0 {
                    for note in self.notes.iter_mut() {
                        note.release(time)
                    }
                }
            }
            ReplayFrame::MousePos(x, y) => {
                // scale the coords from playfield to window
                let pos = self.scaling_helper.scale_coords(Vector2::new(x as f64, y as f64));
                self.mouse_pos = pos;

                for note in self.notes.iter_mut() {
                    note.mouse_move(pos);
                }
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
        if time >= self.end_time {
            manager.completed = true;
            return;
        }

        // if the time is leading in, we dont want to check if any notes have been missed
        if time < 0.0 {return}

        for note in self.notes.iter_mut() {
            let end_time = note.end_time(self.hitwindow_miss);

            // check if note is in hitwindow
            if time - end_time >= 0.0 && !note.was_hit() {
                println!("note missed: {}-{}", time, end_time);

                // check if we missed the current note
                let ntype = note.note_type();
                let flag = match ntype {
                    NoteType::Note => end_time < time,
                    NoteType::Slider 
                    | NoteType::Spinner => end_time <= time,
                    _ => false,
                };

                if flag {
                    match ntype {
                        NoteType::Note => {
                            manager.score.hit_miss(time, end_time);
                            self.draw_points.push((time, note.point_draw_pos(), ScoreHit::Miss));
                        }
                        NoteType::Slider => {
                            let note_time = note.end_time(0.0);
                            // check slider release points
                            // -1.0 for miss hitwindow to indidate it was held to the end (ie, no hitwindow to check)
                            let pts = note.get_points(false, time, (-1.0, self.hitwindow_50, self.hitwindow_100, self.hitwindow_300));
                            self.draw_points.push((time, note.point_draw_pos(), pts));
                            match pts {
                                ScoreHit::None | ScoreHit::Miss => {
                                    manager.score.hit_miss(time, note_time);
                                    manager.hitbar_timings.push((time, time - note_time));
                                }
                                ScoreHit::Other(_, _) => {}
                                pts => {
                                    match pts {
                                        ScoreHit::X300 => manager.score.hit300(time, note_time),
                                        ScoreHit::X100 => manager.score.hit100(time, note_time),
                                        ScoreHit::X50 => manager.score.hit50(time, note_time),
                                        _ => {}
                                    }

                                    // play hitsound
                                    let hitsound = note.get_hitsound();
                                    let hitsamples = note.get_hitsamples().clone();
                                    manager.play_note_sound(note_time, hitsound, hitsamples);


                                    manager.hitbar_timings.push((time, time - note_time));
                                }
                            }
                        }
                        NoteType::Spinner => {}

                        _ => {},
                    }
                }

                // force the note to be misssed
                note.miss(); 
            }
        }
    }
    fn draw(&mut self, args:RenderArgs, manager:&mut IngameManager, list:&mut Vec<Box<dyn Renderable>>) {
        // draw the playfield
        let p1 = self.scaling_helper.scale_coords(Vector2::zero());
        let p2 = self.scaling_helper.scale_coords(FIELD_SIZE);
        let playfield = Rectangle::new(
            [0.2, 0.2, 0.2, 0.5].into(),
            f64::MAX-4.0,
            p1,
            p2 - p1,
            if manager.current_timing_point().kiai {
                Some(Border::new(Color::YELLOW, 2.0))
            } else {None}
        );
        list.push(Box::new(playfield));

        // draw key counter
        self.key_counter.draw(args, list);

        // if this is a replay, we need to draw the replay curser
        if manager.replaying {
            list.push(Box::new(Circle::new(
                Color::RED,
                -999.9,
                self.mouse_pos,
                20.0
            )))
        }


        let time = manager.time();
        for (p_time, pos, pts) in self.draw_points.iter() {
            let mut color;
            match pts {
                ScoreHit::Miss => color = Color::RED,
                ScoreHit::X50  => color = Color::YELLOW,
                ScoreHit::X100 => color = Color::GREEN,
                ScoreHit::X300 => color = Color::new(0.0, 0.7647, 1.0, 1.0),
                ScoreHit::None | ScoreHit::Other(_, _) => continue,
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
        if key == self.settings.left_key {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Left), time, manager);
        }
        if key == self.settings.right_key {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Right), time, manager);
        }
    }
    fn key_up(&mut self, key:piston::Key, manager:&mut IngameManager) {
        if key == piston::Key::LCtrl {
            self.move_playfield = None;
            return;
        }

        let time = manager.time();
        if key == self.settings.left_key {
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::Left), time, manager);
        }
        if key == self.settings.right_key {
            self.handle_replay_frame(ReplayFrame::Release(KeyPress::Right), time, manager);
        }
    }
    
    fn mouse_move(&mut self, pos:Vector2, manager:&mut IngameManager) {
        let time = manager.time();

        if let Some((original, mouse_start)) = self.move_playfield {
            {
                let settings = &mut Settings::get_mut().standard_settings;
                let change = original + (pos - mouse_start);

                settings.playfield_x_offset = change.x;
                settings.playfield_y_offset = change.y;
            }

            self.playfield_changed();
            return;
        }

        // dont continue if this is a replay
        if manager.replaying {return}

        // convert window pos to playfield pos
        let pos = self.scaling_helper.descale_coords(pos);
        self.handle_replay_frame(ReplayFrame::MousePos(pos.x as f32, pos.y as f32), time, manager);
    }
    fn mouse_down(&mut self, btn:piston::MouseButton, manager:&mut IngameManager) {
        if self.settings.ignore_mouse_buttons {return}

        // dont continue if this is a replay
        if manager.replaying {return}

        let time = manager.time();
        if btn == MouseButton::Left {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Left), time, manager);
        }
        if btn == MouseButton::Right {
            self.handle_replay_frame(ReplayFrame::Press(KeyPress::Right), time, manager);
        }
    }
    fn mouse_up(&mut self, btn:piston::MouseButton, manager:&mut IngameManager) {
        if self.settings.ignore_mouse_buttons {return}

        // dont continue if this is a replay
        if manager.replaying {return}

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

            self.playfield_changed();
        }
    }

    fn reset(&mut self, beatmap:&Beatmap) {
        // reset notes
        for note in self.notes.as_mut_slice() {
            note.reset();
        }
        
        // setup hitwindows
        let od = beatmap.metadata.od;
        self.hitwindow_miss = map_difficulty(od, 225.0, 175.0, 125.0); // idk
        self.hitwindow_50   = map_difficulty(od, 200.0, 150.0, 100.0);
        self.hitwindow_100  = map_difficulty(od, 140.0, 100.0, 60.0);
        self.hitwindow_300  = map_difficulty(od, 80.0, 50.0, 20.0);

        self.draw_points.clear();
    }

    fn skip_intro(&mut self, manager: &mut IngameManager) {
        if self.notes.len() == 0 {return}

        let time = self.notes[0].time() - self.notes[0].get_preempt();
        if time < manager.time() {return}

        manager.song.upgrade().unwrap().set_position(time);
    }



    fn timing_bar_things(&self) -> (Vec<(f32,Color)>, (f32,Color)) {
        (vec![
            (self.hitwindow_50, [0.8549, 0.6823, 0.2745, 1.0].into()),
            (self.hitwindow_100, [0.3411, 0.8901, 0.0745, 1.0].into()),
            (self.hitwindow_300, [0.0, 0.7647, 1.0, 1.0].into()),
        ], (self.hitwindow_miss, [0.9, 0.05, 0.05, 1.0].into()))
    }

    fn combo_bounds(&self) -> Rectangle {
        let size = Vector2::new(100.0, 30.0);
        Rectangle::bounds_only(
            Vector2::new(0.0, window_size().y - (size.y + DURATION_HEIGHT)),
            size
        )
    }
}

