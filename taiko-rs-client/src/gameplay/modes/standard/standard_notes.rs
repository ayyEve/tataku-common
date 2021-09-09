use piston::RenderArgs;
use std::f64::consts::PI;
use graphics::CharacterCache;
use ayyeve_piston_ui::render::{Line, Rectangle, Text, fonts::get_font};

use taiko_rs_common::types::ScoreHit;
use crate::render::{Circle, Color, Renderable, Border};
use crate::{window_size, Vector2, helpers::curve::Curve};
use crate::gameplay::{HitObject, map_difficulty, modes::{scale_coords, scale_cs}, defs::*};

const SPINNER_RADIUS:f64 = 200.0;
// const SPINNER_POSITION:Vector2 = Vector2::new(window_size().x / 2.0, window_size().y / 2.0);
const SLIDER_DOT_RADIUS:f64 = 8.0;
const NOTE_BORDER_SIZE:f64 = 2.0;

const CIRCLE_RADIUS_BASE:f64 = 64.0;
const HITWINDOW_CIRCLE_RADIUS:f64 = CIRCLE_RADIUS_BASE * 2.0;
const PREEMPT_MIN:f32 = 450.0;
const NOTE_DEPTH:f64 = -100.0;

// pub const POS_OFFSET:Vector2 = Vector2::new((window_size.x - FIELD_SIZE.x) / 2.0, (window_size.y - FIELD_SIZE.y) / 2.0)


pub trait StandardHitObject: HitObject {
    /// does this object count as a miss if it is not hit?
    fn causes_miss(&self) -> bool; //TODO: might change this to return an enum of "no", "yes". "yes_combo_only"
    fn get_points(&mut self, time:f32, hit_windows:(f32,f32,f32)) -> ScoreHit;

    fn playfield_changed(&mut self);

    fn press(&mut self, _time:f32) {}
    fn release(&mut self, _time:f32) {}
    fn mouse_move(&mut self, pos:Vector2);

    fn get_preempt(&self) -> f32;
    fn point_draw_pos(&self) -> Vector2;
}


// note
#[derive(Clone)]
pub struct StandardNote {
    def: NoteDef,
    pos: Vector2,
    time: f32, // ms
    hit: bool,
    missed: bool,

    /// combo color
    color: Color, 
    /// combo number
    combo_num: u16,

    /// note depth
    base_depth: f64,

    map_time: f32,
    mouse_pos: Vector2,
    radius: f64,
    time_preempt: f32,

    cs: f32,
    ar:f32,

    
    combo_text: Box<Text>,
}
impl StandardNote {
    pub fn new(def:NoteDef, ar:f32, cs:f32, color:Color, combo_num:u16) -> Self {
        let time = def.time;
        let cs_scale = (1.0 - 0.7 * (cs - 5.0) / 5.0) / 2.0;
        let time_preempt = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);
        let base_depth = get_depth(def.time);
        let pos = scale_coords(def.pos);
        let radius = CIRCLE_RADIUS_BASE * scale_cs(cs_scale as f64);

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

            ar,
            cs,
            
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

        // timing circle
        list.push(approach_circle(self.pos, self.radius, self.time - self.map_time, self.time_preempt, self.base_depth));
        // combo number
        list.push(self.combo_text.clone());

        // note
        let mut note = Circle::new(
            self.color,
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
    fn point_draw_pos(&self) -> Vector2 {self.pos}
    fn causes_miss(&self) -> bool {true}
    fn mouse_move(&mut self, pos:Vector2) {self.mouse_pos = pos}
    fn get_preempt(&self) -> f32 {self.time_preempt}

    fn get_points(&mut self, time:f32, hit_windows:(f32,f32,f32)) -> ScoreHit {
        let (hitwindow_miss, hitwindow_100, hitwindow_300) = hit_windows;
        let diff = (time - self.time).abs();
        
        // make sure the cursor is in the radius
        let distance = ((self.pos.x - self.mouse_pos.x).powi(2) + (self.pos.y - self.mouse_pos.y).powi(2)).sqrt();
        if distance > self.radius {return ScoreHit::None}

        if diff < hitwindow_300 {
            self.hit = true;
            ScoreHit::X300
        } else if diff < hitwindow_100 {
            self.hit = true;
            ScoreHit::X100
        } else if diff < hitwindow_miss { // too early, miss
            self.missed = true;
            ScoreHit::Miss
        } else { // way too early, ignore
            ScoreHit::None
        }
    }

    fn playfield_changed(&mut self) {
        let cs_scale = (1.0 - 0.7 * (self.cs - 5.0) / 5.0) / 2.0;
        let time_preempt = map_difficulty(self.ar, 1800.0, 1200.0, PREEMPT_MIN);
        let pos = scale_coords(self.def.pos);
        let radius = CIRCLE_RADIUS_BASE * scale_cs(cs_scale as f64);

        let mut combo_text =  Box::new(Text::new(
            Color::BLACK,
            self.base_depth - 0.0000001,
            pos,
            (radius) as u32,
            format!("{}", self.combo_num),
            get_font("main")
        ));
        center_combo_text(&mut combo_text, Rectangle::bounds_only(
            pos - Vector2::one() * radius / 2.0,
            Vector2::one() * radius,
        ));


        self.time_preempt = time_preempt;
        self.combo_text = combo_text;
        self.pos = pos;
        self.radius = radius;

    }
}



// slider
#[derive(Clone)]
pub struct StandardSlider {
    def: SliderDef,

    /// start pos
    pos: Vector2,
    /// visual end pos
    visual_end_pos: Vector2,
    /// time end pos
    time_end_pos: Vector2,

    /// hit dots. if the slider isnt being held for these
    hit_dots: Vec<SliderDot>,

    /// start time
    time: f32,

    /// curve that defines the slider
    curve: Curve,

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

    /// song's current time
    map_time: f32,

    /// note depth
    base_depth: f64,

    /// 
    time_preempt:f32,


    cs: f32,
    ar: f32,


    combo_text: Box<Text>
}
impl StandardSlider {
    pub fn new(def:SliderDef, curve:Curve, ar:f32, cs:f32, color:Color, combo_num: u16) -> Self {
        let time = def.time;
        let cs_scale = (1.0 - 0.7 * (cs - 5.0) / 5.0) / 2.0;
        let time_preempt = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);
        
        let pos = scale_coords(def.pos);
        let visual_end_pos = scale_coords(curve.position_at_length(curve.length()));
        let time_end_pos = if def.slides % 2 == 1 {visual_end_pos} else {pos};

        let base_depth = get_depth(def.time);
        let radius = CIRCLE_RADIUS_BASE * scale_cs(cs_scale as f64);

        let mut combo_text =  Box::new(Text::new(
            Color::BLACK,
            base_depth - 0.0000001,
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
            base_depth,
            radius,

            pos,
            visual_end_pos,
            time_end_pos,

            time, 
            hit_dots: Vec::new(),
            map_time: 0.0,

            start_checked: false,
            end_checked: false,
            hold_time: 0.0,
            release_time: 0.0,
            mouse_pos: Vector2::zero(),

            cs,
            ar,

            combo_text
        }
    }
}
impl HitObject for StandardSlider {
    fn note_type(&self) -> NoteType {NoteType::Slider}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self,_:f32) -> f32 {self.curve.end_time}
    fn update(&mut self, beatmap_time: f32) {self.map_time = beatmap_time}

    fn draw(&mut self, _args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        if self.time - self.map_time > self.time_preempt || self.curve.end_time < self.map_time {return}

        if self.time - self.map_time > 0.0 {
            // timing circle
            list.push(approach_circle(self.pos, self.radius, self.time - self.map_time, self.time_preempt, self.base_depth));
            // combo number
            list.push(self.combo_text.clone());
        } else {
            // slider ball
            let pos = scale_coords(self.curve.position_at_time(self.map_time));
            let distance = ((pos.x - self.mouse_pos.x).powi(2) + (pos.y - self.mouse_pos.y).powi(2)).sqrt();

            let mut inner = Circle::new(
                self.color,
                self.base_depth - 0.0000001,
                pos,
                self.radius
            );
            inner.border = Some(Border::new(
                Color::WHITE,
                2.0
            ));
            list.push(Box::new(inner));


            let radius = self.radius * 2.0;
            let mut outer = Circle::new(
                Color::TRANSPARENT_WHITE,
                self.base_depth - 0.0000001,
                pos,
                radius
            );
            outer.border = Some(Border::new(
                if self.hold_time > self.release_time && distance <= radius {
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
            list.push(Box::new(Line::new(
                scale_coords(line.p1),
                scale_coords(line.p2),
                self.radius,
                self.base_depth,
                self.color
            )));

            // add a circle to smooth out the corners
            list.push(Box::new(Circle::new(
                self.color,
                self.base_depth,
                scale_coords(line.p2),
                self.radius,
            )))
        }
        
        // start and end circles
        for pos in [self.visual_end_pos, self.pos] {
            let mut c = Circle::new(
                self.color,
                self.base_depth - 0.00000005, // should be above curves but below slider ball
                pos,
                self.radius
            );
            c.border = Some(Border {
                color: Color::BLACK,
                radius: NOTE_BORDER_SIZE
            });
            list.push(Box::new(c));
        }

        // draw hit dots
        // for dot in self.hit_dots.as_slice() {
        //     if dot.done {continue}
        //     renderables.extend(dot.draw());
        // }
    }

    fn reset(&mut self) {
        self.hit_dots.clear();
        self.hold_time = 0.0;
        self.release_time = 0.0;
    }
}
impl StandardHitObject for StandardSlider {
    fn causes_miss(&self) -> bool {false}
    fn point_draw_pos(&self) -> Vector2 {self.pos}
    fn get_preempt(&self) -> f32 {self.time_preempt}
    fn press(&mut self, time:f32) {self.hold_time = time}
    fn release(&mut self, time:f32) {self.release_time = time}
    fn mouse_move(&mut self, pos:Vector2) {self.mouse_pos = pos}

    // called on hit and release
    fn get_points(&mut self, time:f32, (h_miss, h100, h300):(f32,f32,f32)) -> ScoreHit {
        // slider was held to end, no hitwindow to check
        if h_miss == -1.0 {
            let distance = ((self.time_end_pos.x - self.mouse_pos.x).powi(2) + (self.time_end_pos.y - self.mouse_pos.y).powi(2)).sqrt();

            if distance > self.radius * 2.0 {println!("slider end miss (out of radius)")}
            if self.hold_time < self.release_time {println!("slider end miss (not held)")}

            return if distance > self.radius * 2.0 || self.hold_time < self.release_time {
                ScoreHit::Miss
            } else {
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


            // if already hit, return None
            if self.start_checked {return ScoreHit::None}
            
            // start wasnt hit yet, set it to true
            self.start_checked = true;
            
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
        } else {
            ScoreHit::Miss
        }

        // self.hit_dots.push(SliderDot::new(time, self.speed));
        // ScoreHit::Other(100, false)
    }


    fn playfield_changed(&mut self) {
        let cs_scale = (1.0 - 0.7 * (self.cs - 5.0) / 5.0) / 2.0;
        let pos = scale_coords(self.def.pos);
        let radius = CIRCLE_RADIUS_BASE * scale_cs(cs_scale as f64);
        self.visual_end_pos = scale_coords(self.curve.position_at_length(self.curve.length()));
        self.time_end_pos = if self.def.slides % 2 == 1 {self.visual_end_pos} else {self.pos};
    

        let mut combo_text =  Box::new(Text::new(
            Color::BLACK,
            self.base_depth - 0.0000001,
            pos,
            (radius) as u32,
            format!("{}", self.combo_num),
            get_font("main")
        ));
        center_combo_text(&mut combo_text, Rectangle::bounds_only(
            pos - Vector2::one() * radius / 2.0,
            Vector2::one() * radius,
        ));

        self.combo_text = combo_text;
        self.pos = pos;
        self.radius = radius;
    }
}

/// helper struct for drawing hit slider points
#[derive(Clone, Copy)]
struct SliderDot {
    time: f64,
    pos: Vector2,
    pub done: bool
}
impl SliderDot {
    pub fn new(time:f64, pos:Vector2) -> SliderDot {
        SliderDot {
            time,
            pos,
            done: false
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
#[derive(Clone, Copy)]
pub struct StandardSpinner {
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
    pub fn new(time:f32, end_time:f32) -> Self {
        Self {
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
        if beatmap_time >= self.time && beatmap_time <= self.end_time {
            let mut diff = 0.0;
            let pos_diff = self.mouse_pos - (window_size() / 2.0);
            let mouse_angle = pos_diff.y.atan2(pos_diff.x);
            if self.holding {diff = self.last_rotation_val - mouse_angle}

            self.last_rotation_val = mouse_angle;
            if diff.abs() > PI {diff = 0.0}
            self.rotation_velocity = lerp(-diff, self.rotation_velocity, 0.05 * (beatmap_time - self.last_update) as f64);
            self.rotation += self.rotation_velocity;
        }

        self.last_update = beatmap_time;
    }
    fn draw(&mut self, _args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        if !(self.last_update >= self.time && self.last_update <= self.end_time) {return}

        let pos = window_size() / 2.0;
        // bg circle
        let mut bg = Circle::new(
            Color::YELLOW,
            -10.0,
            pos,
            SPINNER_RADIUS
        );
        bg.border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE));
        list.push(Box::new(bg));

        // draw another circle on top which increases in radius as the counter gets closer to the reqired
        let mut fg = Circle::new(
            Color::WHITE,
            -11.0,
            pos,
            SPINNER_RADIUS * (self.rotations_completed as f64 / self.rotations_required as f64)
        );
        fg.border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE));
        list.push(Box::new(fg));

        // draw line to show rotation
        {
            let p2 = pos + Vector2::new(self.rotation.cos(), self.rotation.sin()) * SPINNER_RADIUS;
            list.push(Box::new(Line::new(
                pos,
                p2,
                5.0,
                -20.0,
                Color::GREEN
            )));
        }
            

        //TODO: draw a counter
    }

    fn reset(&mut self) {
        self.rotation = 0.0;
        self.rotations_completed = 0;
    }
}
impl StandardHitObject for StandardSpinner {
    fn get_preempt(&self) -> f32 {0.0}
    fn point_draw_pos(&self) -> Vector2 {Vector2::zero()} //TODO
    fn causes_miss(&self) -> bool {self.rotations_completed < self.rotations_required} // if the spinner wasnt completed in time, cause a miss

    fn get_points(&mut self, _time:f32, _:(f32,f32,f32)) -> ScoreHit {
        ScoreHit::Other(100, false)
    }

    fn press(&mut self, time:f32) {
        if time >= self.time && time <= self.end_time {
            self.holding = true;
        }
    }
    fn release(&mut self, _time:f32) {
        self.holding = false;
    }
    fn mouse_move(&mut self, pos:Vector2) {
        self.mouse_pos = pos;
    }

    fn playfield_changed(&mut self) {
        
    } 
}



fn lerp(target:f64, current: f64, factor:f64) -> f64 {
    current + (target - current) * factor
}

fn get_depth(time:f32) -> f64 {
    (NOTE_DEPTH + time as f64) / 1000.0
}
fn approach_circle(pos:Vector2, radius:f64, time_diff: f32, time_preempt:f32, depth: f64) -> Box<Circle> {
    let scale = scale_cs(1.0);

    let mut c = Circle::new(
        Color::TRANSPARENT_WHITE,
        depth - 100.0,
        pos,
        radius + (time_diff as f64 / time_preempt as f64) * (HITWINDOW_CIRCLE_RADIUS * scale)
    );
    c.border = Some(Border::new(Color::WHITE, NOTE_BORDER_SIZE * scale));
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