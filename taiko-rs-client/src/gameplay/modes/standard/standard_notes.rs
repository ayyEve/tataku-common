use piston::RenderArgs;
use std::f64::consts::PI;
use graphics::CharacterCache;

use taiko_rs_common::types::ScoreHit;
use crate::gameplay::modes::ScalingHelper;
use crate::{window_size, Vector2, helpers::curve::Curve};
use crate::gameplay::{HitObject, map_difficulty, defs::*};
use crate::render::{Circle, Color, Renderable, Border, Line, Rectangle, Text, fonts::get_font};

const SPINNER_RADIUS:f64 = 200.0;
const SLIDER_DOT_RADIUS:f64 = 8.0;
const NOTE_BORDER_SIZE:f64 = 2.0;

pub const CIRCLE_RADIUS_BASE:f64 = 64.0;
const HITWINDOW_CIRCLE_RADIUS:f64 = CIRCLE_RADIUS_BASE * 2.0;
const PREEMPT_MIN:f32 = 450.0;

// pub const POS_OFFSET:Vector2 = Vector2::new((window_size.x - FIELD_SIZE.x) / 2.0, (window_size.y - FIELD_SIZE.y) / 2.0)


pub trait StandardHitObject: HitObject {
    /// does this object count as a miss if it is not hit?
    fn causes_miss(&self) -> bool; //TODO: might change this to return an enum of "no", "yes". "yes_combo_only" 
    fn get_points(&mut self, is_press:bool, time:f32, hit_windows:(f32,f32,f32,f32)) -> ScoreHit;
    /// return negative for combo break
    fn pending_combo(&mut self) -> i8 {0}

    fn playfield_changed(&mut self, new_scale: &ScalingHelper);

    fn press(&mut self, _time:f32) {}
    fn release(&mut self, _time:f32) {}
    fn mouse_move(&mut self, pos:Vector2);

    fn get_preempt(&self) -> f32;
    fn point_draw_pos(&self) -> Vector2;

    fn was_hit(&self) -> bool;

    fn miss(&mut self);
    fn get_hitsound(&self) -> u8;
    fn get_hitsamples(&self) -> HitSamples;
    fn get_sound_queue(&mut self) -> Vec<(f32, u8, HitSamples, Option<String>)> {vec![]}
}


// note
#[derive(Clone)]
pub struct StandardNote {
    /// note definition
    def: NoteDef,
    /// note position
    pos: Vector2,
    /// note time in ms
    time: f32,

    /// was the note hit?
    hit: bool,
    /// was the note missed?
    missed: bool,

    /// combo color
    color: Color, 
    /// combo number
    combo_num: u16,

    /// note depth
    base_depth: f64,
    /// note radius (scaled by cs and size)
    radius: f64,
    /// when the hitcircle should start being drawn
    time_preempt: f32,
    /// what is the scaling value? needed for approach circle
    // (lol)
    scaling_scale: f64,
    
    /// combo num text cache
    combo_text: Box<Text>,


    /// current map time
    map_time: f32,
    /// current mouse pos
    mouse_pos: Vector2,
}
impl StandardNote {
    pub fn new(def:NoteDef, ar:f32, color:Color, combo_num:u16, scaling_helper: &ScalingHelper, base_depth:f64) -> Self {
        let time = def.time;
        let time_preempt = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);

        let pos = scaling_helper.scale_coords(def.pos);
        let radius = CIRCLE_RADIUS_BASE * scaling_helper.scaled_cs;

        let mut combo_text =  Box::new(Text::new(
            Color::BLACK,
            base_depth - 0.0000001,
            pos,
            (radius) as u32,
            format!("{}", combo_num),
            get_font("main")
        ));
        center_combo_text(&mut combo_text, Rectangle::bounds_only(
            pos - Vector2::one() * radius / 2.0,
            Vector2::one() * radius,
        ));

        Self {
            def,
            pos,
            time, 
            base_depth,
            color,
            combo_num,
            
            hit: false,
            missed: false,

            map_time: 0.0,
            mouse_pos: Vector2::zero(),

            time_preempt,
            radius,
            scaling_scale: scaling_helper.scale,
            
            combo_text
        }
    }
}
impl HitObject for StandardNote {
    fn note_type(&self) -> NoteType {NoteType::Note}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self, hw_miss:f32) -> f32 {self.time + hw_miss}
    fn update(&mut self, beatmap_time: f32) {self.map_time = beatmap_time}

    fn draw(&mut self, _args:RenderArgs, list:&mut Vec<Box<dyn Renderable>>) {
        if self.time - self.map_time > self.time_preempt || self.time - self.map_time < 0.0 || self.hit {return}

        let alpha = (1.0 - ((self.time - (self.time_preempt * (2.0/3.0))) - self.map_time) / (self.time_preempt * (1.0/3.0))).clamp(0.0, 1.0);

        // timing circle
        list.push(approach_circle(self.pos, self.radius, self.time - self.map_time, self.time_preempt, self.base_depth, self.scaling_scale, alpha));


        // combo number
        self.combo_text.color.alpha(alpha);
        list.push(self.combo_text.clone());

        // note
        let mut note = Circle::new(
            self.color.alpha(alpha),
            self.base_depth,
            self.pos,
            self.radius
        );
        note.border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE));
        list.push(Box::new(note));
    }

    fn reset(&mut self) {
        self.hit = false;
        self.missed = false;
    }
}
impl StandardHitObject for StandardNote {
    fn miss(&mut self) {self.missed = true}
    fn was_hit(&self) -> bool {self.hit || self.missed}
    fn get_hitsamples(&self) -> HitSamples {self.def.hitsamples.clone()}
    fn get_hitsound(&self) -> u8 {self.def.hitsound}
    fn point_draw_pos(&self) -> Vector2 {self.pos}
    fn causes_miss(&self) -> bool {true}
    fn mouse_move(&mut self, pos:Vector2) {self.mouse_pos = pos}
    fn get_preempt(&self) -> f32 {self.time_preempt}

    fn get_points(&mut self, _is_press:bool, time:f32, (hitwindow_miss, hitwindow_50, hitwindow_100, hitwindow_300):(f32,f32,f32,f32)) -> ScoreHit {
        let diff = (time - self.time).abs();
        
        // make sure the cursor is in the radius
        let distance = ((self.pos.x - self.mouse_pos.x).powi(2) + (self.pos.y - self.mouse_pos.y).powi(2)).sqrt();
        if distance > self.radius {
            println!("mouse too far: {} > {}", distance, self.radius);
            return ScoreHit::None
        }

        if diff < hitwindow_300 {
            self.hit = true;
            ScoreHit::X300
        } else if diff < hitwindow_100 {
            self.hit = true;
            ScoreHit::X100
        } else if diff < hitwindow_50 {
            self.hit = true;
            ScoreHit::X50
        } else if diff < hitwindow_miss { // too early, miss
            self.missed = true;
            ScoreHit::Miss
        } else { // way too early, ignore
            ScoreHit::None
        }
    }

    fn playfield_changed(&mut self, new_scale: &ScalingHelper) {
        self.pos = new_scale.scale_coords(self.def.pos);
        self.radius = CIRCLE_RADIUS_BASE * new_scale.scaled_cs;
        self.scaling_scale = new_scale.scale;

        let mut combo_text =  Box::new(Text::new(
            Color::BLACK,
            self.base_depth - 0.0000001,
            self.pos,
            self.radius as u32,
            format!("{}", self.combo_num),
            get_font("main")
        ));
        center_combo_text(&mut combo_text, Rectangle::bounds_only(
            self.pos - Vector2::one() * self.radius / 2.0,
            Vector2::one() * self.radius,
        ));

        self.combo_text = combo_text;
    }
}



// slider
#[derive(Clone)]
pub struct StandardSlider {
    /// slider definition for this slider
    def: SliderDef,
    /// curve that defines the slider
    curve: Curve,

    /// start pos
    pos: Vector2,
    /// visual end pos
    visual_end_pos: Vector2,
    /// time end pos
    time_end_pos: Vector2,

    /// hit dots. if the slider isnt being held for these
    hit_dots: Vec<SliderDot>,

    /// used for repeat sliders
    pending_combo: i8,

    /// start time
    time: f32,
    /// what is the current sound index?
    sound_index: usize,
    /// how many slides have been completed?
    slides_complete: u64,
    /// used to check if a slide has been completed
    moving_forward: bool,
    /// song's current time
    map_time: f32,

    /// combo color
    color: Color,
    /// combo number
    combo_num: u16,
    /// note size
    radius: f64,
    
    /// was the start checked?
    start_checked: bool,
    /// was the release checked?
    end_checked: bool,

    /// hold start
    hold_time: f32, 
    /// hold end
    release_time: f32,
    /// stored mouse pos
    mouse_pos: Vector2,

    /// slider curve depth
    slider_depth: f64,
    /// start/end circle depth
    circle_depth: f64,
    /// when should the note start being drawn (specifically the )
    time_preempt:f32,

    /// combo text cache, probably not needed but whatever
    combo_text: Box<Text>,

    /// list of sounds waiting to be played (used by repeat and slider dot sounds)
    /// (time, hitsound, samples, override sample name)
    sound_queue: Vec<(f32, u8, HitSamples, Option<String>)>,

    /// scaling helper, should greatly improve rendering speed due to locking
    scaling_helper: ScalingHelper,

    /// is the mouse in a good state for sliding? (pos + key down)
    sliding_ok: bool,

    /// cached slider ball pos
    slider_ball_pos: Vector2,
}
impl StandardSlider {
    pub fn new(def:SliderDef, curve:Curve, ar:f32, color:Color, combo_num: u16, scaling_helper:ScalingHelper, slider_depth:f64, circle_depth:f64) -> Self {
        let time = def.time;
        let time_preempt = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);
        
        let pos = scaling_helper.scale_coords(def.pos);
        let visual_end_pos = scaling_helper.scale_coords(curve.position_at_length(curve.length()));
        let time_end_pos = if def.slides % 2 == 1 {visual_end_pos} else {pos};
        let radius = CIRCLE_RADIUS_BASE * scaling_helper.scaled_cs;

        let mut combo_text =  Box::new(Text::new(
            Color::BLACK,
            circle_depth - 0.0000001,
            pos,
            radius as u32,
            format!("{}", combo_num),
            get_font("main")
        ));
        center_combo_text(&mut combo_text, Rectangle::bounds_only(
            pos - Vector2::one() * radius / 2.0,
            Vector2::one() * radius,
        ));

        Self {
            def,
            curve,
            color,
            combo_num,
            time_preempt,
            slider_depth,
            circle_depth,
            radius,

            pos,
            visual_end_pos,
            time_end_pos,

            time, 
            pending_combo: 0,
            sound_index: 0,
            slides_complete: 0,
            moving_forward: true,
            hit_dots: Vec::new(),
            map_time: 0.0,

            start_checked: false,
            end_checked: false,
            hold_time: 0.0,
            release_time: 0.0,
            mouse_pos: Vector2::zero(),

            combo_text,
            sound_queue: Vec::new(),

            scaling_helper,
            sliding_ok: false,
            slider_ball_pos: Vector2::zero()
        }
    }
}
impl HitObject for StandardSlider {
    fn note_type(&self) -> NoteType {NoteType::Slider}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self,_:f32) -> f32 {self.curve.end_time}
    fn update(&mut self, beatmap_time: f32) {
        self.map_time = beatmap_time;

        // check sliding ok
        self.slider_ball_pos = self.scaling_helper.scale_coords(self.curve.position_at_time(beatmap_time));
        let distance = ((self.slider_ball_pos.x - self.mouse_pos.x).powi(2) + (self.slider_ball_pos.y - self.mouse_pos.y).powi(2)).sqrt();
        self.sliding_ok = self.hold_time > self.release_time && distance <= self.radius * 2.0;


        // find out if a slide has been completed
        let pos = (beatmap_time - self.time) / (self.curve.length() / self.def.slides as f32);
        if self.time - beatmap_time > self.time_preempt || self.curve.end_time < beatmap_time {return}

        let current_moving_forwards = pos % 2.0 <= 1.0;
        if current_moving_forwards != self.moving_forward {
            // direction changed
            self.moving_forward = current_moving_forwards;
            self.slides_complete += 1;
            // println!("repeat {} started", self.slides_complete);

            // increment index
            self.sound_index += 1;

            // check cursor
            if self.sliding_ok {
                // increment pending combo
                self.pending_combo += 1;
                self.sound_queue.push((
                    beatmap_time,
                    self.get_hitsound(),
                    self.get_hitsamples().clone(),
                    None
                ));
            } else {
                // set it to negative, we broke combo
                self.pending_combo = -1;
            }

        }
    }

    fn draw(&mut self, _args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        if self.time - self.map_time > self.time_preempt || self.curve.end_time < self.map_time {return}

        // let alpha = (self.time_preempt / 4.0) / ((self.time - self.time_preempt / 4.0) - self.map_time).clamp(0.0, 1.0);
        let alpha = (1.0 - ((self.time - (self.time_preempt * (2.0/3.0))) - self.map_time) / (self.time_preempt * (1.0/3.0))).clamp(0.0, 1.0);

        // combo number
        self.combo_text.color.alpha(alpha);
        list.push(self.combo_text.clone());

        if self.time - self.map_time > 0.0 {
            // timing circle
            list.push(approach_circle(self.pos, self.radius, self.time - self.map_time, self.time_preempt, self.circle_depth, self.scaling_helper.scale, alpha));
            // combo number
            list.push(self.combo_text.clone());

        } else {
            // slider ball
            let mut inner = Circle::new(
                self.color,
                self.circle_depth - 0.0000001,
                self.slider_ball_pos,
                self.radius
            );
            inner.border = Some(Border::new(
                Color::WHITE,
                2.0
            ));
            list.push(Box::new(inner));


            let mut outer = Circle::new(
                Color::TRANSPARENT_WHITE,
                self.circle_depth - 0.0000001,
                self.slider_ball_pos,
                self.radius* 2.0
            );
            outer.border = Some(Border::new(
                if self.sliding_ok {
                    Color::GREEN
                } else {
                    Color::RED
                },
                2.0
            ));
            list.push(Box::new(outer));
        }


        // curves
        list.reserve(self.curve.path.len() * 2);
        for i in 0..self.curve.path.len() {
            let line = self.curve.path[i];

            let p1 = self.scaling_helper.scale_coords(line.p1);
            let p2 = self.scaling_helper.scale_coords(line.p2);
            list.push(Box::new(Line::new(
                p1,
                p2,
                self.radius,
                self.slider_depth,
                self.color.alpha(alpha)
            )));

            // add a circle to smooth out the corners
            list.push(Box::new(Circle::new(
                self.color.alpha(alpha),
                self.slider_depth,
                p2,
                self.radius,
            )))
        }
        
        // start and end circles
        let repeats = self.def.slides > 1;
        let repeat_diff = self.def.slides - self.slides_complete;

        // end pos
        let mut c = Circle::new(
            self.color.alpha(alpha),
            self.circle_depth, // should be above curves but below slider ball
            self.visual_end_pos,
            self.radius
        );
        c.border = Some(Border::new(
            if repeats && repeat_diff > 1 {
                Color::RED
            } else {
                Color::BLACK
            },
            NOTE_BORDER_SIZE
        ));
        list.push(Box::new(c));

        // start pos
        let mut c = Circle::new(
            self.color.alpha(alpha),
            self.circle_depth, // should be above curves but below slider ball
            self.pos,
            self.radius
        );
        c.border = Some(Border::new(
            if repeats && repeat_diff > 2 {
                Color::RED
            } else {
                Color::BLACK
            },
            NOTE_BORDER_SIZE
        ));
        list.push(Box::new(c));


        // draw hit dots
        // for dot in self.hit_dots.as_slice() {
        //     if dot.done {continue}
        //     renderables.extend(dot.draw());
        // }
    }

    fn reset(&mut self) {
        self.hit_dots.clear();
        self.sound_queue.clear();

        self.map_time = 0.0;
        self.hold_time = 0.0;
        self.release_time = 0.0;
        self.start_checked = false;
        self.end_checked = false;
        
        self.pending_combo = 0;
        self.sound_index = 0;
        self.slides_complete = 0;
        self.moving_forward = true;
    }
}
impl StandardHitObject for StandardSlider {
    fn miss(&mut self) {self.end_checked = true}
    fn was_hit(&self) -> bool {self.end_checked}
    fn get_hitsamples(&self) -> HitSamples {
        let mut samples = self.def.hitsamples.clone();
        let [normal_set, addition_set] = self.def.edge_sets[self.sound_index.min(self.def.edge_sets.len() - 1)];
        samples.normal_set = normal_set;
        samples.addition_set = addition_set;

        samples
    }
    fn get_hitsound(&self) -> u8 {
        // println!("{}: getting hitsound at index {}/{}", self.time, self.sound_index, self.def.edge_sounds.len() - 1);
        self.def.edge_sounds[self.sound_index.min(self.def.edge_sounds.len() - 1)]
    }
    fn causes_miss(&self) -> bool {false}
    fn point_draw_pos(&self) -> Vector2 {
        if self.end_checked {self.time_end_pos}
        else {self.pos}
    }
    fn get_preempt(&self) -> f32 {self.time_preempt}
    fn press(&mut self, time:f32) {self.hold_time = time}
    fn release(&mut self, time:f32) {self.release_time = time}
    fn mouse_move(&mut self, pos:Vector2) {self.mouse_pos = pos}

    // called on hit and release
    fn get_points(&mut self, is_press:bool, time:f32, (h_miss, h50, h100, h300):(f32,f32,f32,f32)) -> ScoreHit {
        // if slider was held to end, no hitwindow to check
        if h_miss == -1.0 {
            let distance = ((self.time_end_pos.x - self.mouse_pos.x).powi(2) + (self.time_end_pos.y - self.mouse_pos.y).powi(2)).sqrt();

            if distance > self.radius * 2.0 {println!("slider end miss (out of radius)")}
            if self.hold_time < self.release_time {println!("slider end miss (not held)")}

            return if distance > self.radius * 2.0 || self.hold_time < self.release_time {
                ScoreHit::X100
            } else {
                self.sound_index = self.def.edge_sounds.len() - 1;
                ScoreHit::X300
            }
        }

        // make sure the cursor is in the radius
        let distance = ((self.pos.x - self.mouse_pos.x).powi(2) + (self.pos.y - self.mouse_pos.y).powi(2)).sqrt();
        // outside the radius, but we dont want it to consume the object
        if distance > self.radius {return ScoreHit::None}
        
        let judgement_time: f32;

        // check press
        if time > self.time - h_miss && time < self.time + h_miss {
            // within starting time frame

            // if already hit, or this is a release, return None
            if self.start_checked || !is_press {return ScoreHit::None}
            
            // start wasnt hit yet, set it to true
            self.start_checked = true;
            // self.sound_index += 1;
            
            // set the judgement time to our start time
            judgement_time = self.time;
        } else 

        // check release
        if time > self.curve.end_time - h_miss && time < self.curve.end_time + h_miss {
            // within ending time frame

            // if already hit, return None
            if self.end_checked {return ScoreHit::None}
            
            // start wasnt hit yet, set it to true
            self.end_checked = true;

            // make sure the last hitsound in the list is played
            self.sound_index = self.def.edge_sounds.len() - 1;
            
            // set the judgement time to our end time
            judgement_time = self.curve.end_time;
        } 
        // not in either time frame, exit
        else {
            return ScoreHit::None;
        }

        // at this point, assume we want to return points
        // get the points
        let diff = (time - judgement_time).abs();

        if diff < h300 {
            ScoreHit::X300
        } else if diff < h100 {
            ScoreHit::X100
        } else if diff < h50 {
            ScoreHit::X50
        } else {
            ScoreHit::Miss
        }

        // self.hit_dots.push(SliderDot::new(time, self.speed));
        // ScoreHit::Other(100, false)
    }


    fn get_sound_queue(&mut self) -> Vec<(f32, u8, HitSamples, Option<String>)> {
        std::mem::take(&mut self.sound_queue)
    }
    fn pending_combo(&mut self) -> i8 {
        std::mem::take(&mut self.pending_combo)
    }


    fn playfield_changed(&mut self, new_scale: &ScalingHelper) {
        self.scaling_helper = new_scale.clone();
        self.pos = new_scale.scale_coords(self.def.pos);
        self.radius = CIRCLE_RADIUS_BASE * new_scale.scaled_cs;
        self.visual_end_pos = new_scale.scale_coords(self.curve.position_at_length(self.curve.length()));
        self.time_end_pos = if self.def.slides % 2 == 1 {self.visual_end_pos} else {self.pos};
        
        let mut combo_text =  Box::new(Text::new(
            Color::BLACK,
            self.circle_depth - 0.0000001,
            self.pos,
            self.radius as u32,
            format!("{}", self.combo_num),
            get_font("main")
        ));
        center_combo_text(&mut combo_text, Rectangle::bounds_only(
            self.pos - Vector2::one() * self.radius / 2.0,
            Vector2::one() * self.radius,
        ));

        self.combo_text = combo_text;
    }
}

/// helper struct for drawing hit slider points
#[derive(Clone, Copy)]
struct SliderDot {
    time: f64,
    pos: Vector2
}
impl SliderDot {
    pub fn new(time:f64, pos:Vector2) -> SliderDot {
        SliderDot {
            time,
            pos
        }
    }
    pub fn update(&mut self, _beatmap_time:f64) {}
    pub fn draw(&self, list:&mut Vec<Box<dyn Renderable>>) {
        let mut c = Circle::new(
            Color::YELLOW,
            -100.0,
            self.pos,
            SLIDER_DOT_RADIUS
        );
        c.border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE/2.0));

        list.push(Box::new(c));
        // [
        //     Box::new(c),
        //     // "hole punch"
        //     Box::new(Circle::new(
        //         BAR_COLOR,
        //         0.0,
        //         Vector2::new(self.pos.x, HIT_POSITION.y),
        //         SLIDER_DOT_RADIUS
        //     )),
        // ]
    }
}


// spinner
#[derive(Clone)]
pub struct StandardSpinner {
    def: SpinnerDef,
    pos: Vector2,
    time: f32, // ms
    end_time: f32, // ms

    /// current angle of the spinner
    rotation: f64,
    /// how fast the spinner is spinning
    rotation_velocity: f64,

    /// what was the last rotation value?
    last_rotation_val: f64,
    /// how many rotations is needed to pass this spinner
    rotations_required: u16,
    /// how many rotations have been completed?
    rotations_completed: u16,

    /// should we count mouse movements?
    holding: bool,

    mouse_pos: Vector2,

    last_update: f32
}
impl StandardSpinner {
    pub fn new(def: SpinnerDef, scaling_helper: &ScalingHelper) -> Self {
        let time = def.time;
        let end_time = def.end_time;
        Self {
            pos: scaling_helper.window_size / 2.0,
            def,
            time, 
            end_time,

            holding: false,
            rotation: 0.0,
            rotation_velocity: 0.0,
            last_rotation_val: 0.0,

            rotations_required: 0,
            rotations_completed: 0,
            mouse_pos: Vector2::zero(),

            last_update: 0.0
        }
    }
}
impl HitObject for StandardSpinner {
    fn time(&self) -> f32 {self.time}
    fn end_time(&self,_:f32) -> f32 {self.end_time}
    fn note_type(&self) -> NoteType {NoteType::Spinner}

    fn update(&mut self, beatmap_time: f32) {
        let mut diff = 0.0;
        let pos_diff = self.mouse_pos - self.pos;
        let mouse_angle = pos_diff.y.atan2(pos_diff.x);

        if beatmap_time >= self.time && beatmap_time <= self.end_time {
            if self.holding {
                diff = self.last_rotation_val - mouse_angle;
            }
            if diff.abs() > PI {diff = 0.0}
            self.rotation_velocity = lerp(-diff, self.rotation_velocity, 0.005 * (beatmap_time - self.last_update) as f64);
            self.rotation += self.rotation_velocity * (beatmap_time - self.last_update) as f64;

            // println!("rotation: {}, diff: {}", self.rotation, diff);
        }

        self.last_rotation_val = mouse_angle;
        self.last_update = beatmap_time;
    }
    fn draw(&mut self, _args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        if !(self.last_update >= self.time && self.last_update <= self.end_time) {return}

        // bg circle
        let mut bg = Circle::new(
            Color::YELLOW,
            -10.0,
            self.pos,
            SPINNER_RADIUS
        );
        bg.border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE));
        list.push(Box::new(bg));

        // draw another circle on top which increases in radius as the counter gets closer to the reqired
        let mut fg = Circle::new(
            Color::WHITE,
            -11.0,
            self.pos,
            SPINNER_RADIUS * (self.rotations_completed as f64 / self.rotations_required as f64)
        );
        fg.border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE));
        list.push(Box::new(fg));

        // draw line to show rotation
        {
            let p2 = self.pos + Vector2::new(self.rotation.cos(), self.rotation.sin()) * SPINNER_RADIUS;
            list.push(Box::new(Line::new(
                self.pos,
                p2,
                5.0,
                -20.0,
                Color::GREEN
            )));
        }
        

        // draw a counter
        let rpm = (self.rotation_velocity * 1000.0 * 60.0) / (2.0 * PI);
        let mut txt = Text::new(
            Color::BLACK,
            -999.9,
            Vector2::zero(),
            30,
            format!("{:.0}rpm", rpm.abs()),
            get_font("main")
        );
        txt.center_text(Rectangle::bounds_only(
            Vector2::new(0.0, self.pos.y + 50.0),
            Vector2::new(self.pos.x * 2.0, 50.0)
        ));
        list.push(Box::new(txt));
    }

    fn reset(&mut self) {
        self.holding = false;
        self.rotation = 0.0;
        self.rotation_velocity = 0.0;
        self.rotations_completed = 0;
    }
}
impl StandardHitObject for StandardSpinner {
    fn miss(&mut self) {}
    fn was_hit(&self) -> bool {self.last_update >= self.end_time} 
    fn get_hitsamples(&self) -> HitSamples {self.def.hitsamples.clone()}
    fn get_hitsound(&self) -> u8 {self.def.hitsound}
    fn get_preempt(&self) -> f32 {0.0}
    fn point_draw_pos(&self) -> Vector2 {Vector2::zero()} //TODO
    fn causes_miss(&self) -> bool {self.rotations_completed < self.rotations_required} // if the spinner wasnt completed in time, cause a miss

    fn get_points(&mut self, _is_press:bool, _:f32, _:(f32,f32,f32,f32)) -> ScoreHit {
        ScoreHit::Other(100, false)
    }

    fn press(&mut self, _time:f32) {
        self.holding = true;
    }
    fn release(&mut self, _time:f32) {
        self.holding = false;
    }
    fn mouse_move(&mut self, pos:Vector2) {
        self.mouse_pos = pos;
    }

    fn playfield_changed(&mut self, _new_scale: &ScalingHelper) {
        
    } 
}



fn lerp(target:f64, current: f64, factor:f64) -> f64 {
    current + (target - current) * factor
}

fn approach_circle(pos:Vector2, radius:f64, time_diff:f32, time_preempt:f32, depth:f64, scale:f64, alpha: f32) -> Box<Circle> {

    let mut c = Circle::new(
        Color::TRANSPARENT_WHITE,
        depth - 100.0,
        pos,
        radius + (time_diff as f64 / time_preempt as f64) * (HITWINDOW_CIRCLE_RADIUS * scale)
    );
    c.border = Some(Border::new(Color::WHITE.alpha(alpha), NOTE_BORDER_SIZE * scale));
    Box::new(c)
}

fn center_combo_text(text:&mut Box<Text>, rect:Rectangle) {
    let mut text_size = Vector2::zero();
    let mut font = text.font.lock();

    for _ch in text.text.chars() {
        let character = font.character(text.font_size, _ch).unwrap();
        text_size.x += character.advance_width();
        text_size.y = text_size.y.max(character.offset[1]); //character.advance_height();
    }

    text.pos = rect.pos + (rect.size - text_size)/2.0
         + Vector2::new(0.0, text_size.y);
}