use ayyeve_piston_ui::render::Line;
use piston::RenderArgs;

use crate::{WINDOW_SIZE, Vector2};
use crate::helpers::slider::Curve;
use taiko_rs_common::types::ScoreHit;
use crate::render::{Circle, Color, Renderable, Border};
use crate::gameplay::{HitObject, map_difficulty, modes::FIELD_SIZE, defs::*};

const SPINNER_RADIUS:f64 = 200.0;
const SPINNER_POSITION:Vector2 = Vector2::new(WINDOW_SIZE.x / 2.0, WINDOW_SIZE.y / 2.0);
const SLIDER_DOT_RADIUS:f64 = 8.0;
const NOTE_BORDER_SIZE:f64 = 2.0;

const CIRCLE_RADIUS_BASE:f64 = 64.0;
const HITWINDOW_CIRCLE_RADIUS:f64 = CIRCLE_RADIUS_BASE * 2.0;
const PREEMPT_MIN:f32 = 450.0;
const NOTE_DEPTH:f64 = -100.0;

pub const POS_OFFSET:Vector2 = Vector2::new((WINDOW_SIZE.x - FIELD_SIZE.x) / 2.0, (WINDOW_SIZE.y - FIELD_SIZE.y) / 2.0);


pub trait StandardHitObject: HitObject {
    /// does this object count as a miss if it is not hit?
    fn causes_miss(&self) -> bool; //TODO: might change this to return an enum of "no", "yes". "yes_combo_only"
    fn get_points(&mut self, time:f32, hit_windows:(f32,f32,f32)) -> ScoreHit;

    fn press(&mut self, _time:f32) {}
    fn release(&mut self, _time:f32) {}
    fn mouse_move(&mut self, pos:Vector2);

    fn get_preempt(&self) -> f32;
}


// note
#[derive(Clone, Copy)]
pub struct StandardNote {
    pos: Vector2,
    time: f32, // ms
    hit_time: f32,
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
    time_preempt: f32
}
impl StandardNote {
    pub fn new(def:NoteDef, ar:f32, cs:f32, color:Color, combo_num:u16) -> Self {
        let cs_scale = (1.0 - 0.7 * (cs - 5.0) / 5.0) / 2.0;
        let time_preempt = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);

        Self {
            pos: POS_OFFSET + def.pos,
            time: def.time, 
            base_depth: get_depth(def.time),
            color,
            combo_num,
            
            hit: false,
            hit_time: 0.0,
            missed: false,

            map_time: 0.0,
            mouse_pos: Vector2::zero(),

            time_preempt,
            radius: CIRCLE_RADIUS_BASE * cs_scale as f64
        }
    }
}
impl HitObject for StandardNote {
    fn note_type(&self) -> NoteType {NoteType::Note}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self, hw_miss:f32) -> f32 {self.time + hw_miss}
    fn update(&mut self, beatmap_time: f32) {self.map_time = beatmap_time}

    fn draw(&mut self, _args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        if self.time - self.map_time > self.time_preempt || self.time - self.map_time < 0.0 {return}

        // timing circle
        list.push(timing_circle(self.pos, self.radius, self.time - self.map_time, self.time_preempt, self.base_depth));

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
        self.hit_time = 0.0;
    }
}
impl StandardHitObject for StandardNote {
    fn causes_miss(&self) -> bool {true}
    fn mouse_move(&mut self, pos:Vector2) {self.mouse_pos = pos}
    fn get_preempt(&self) -> f32 {self.time_preempt}

    fn get_points(&mut self, time:f32, hit_windows:(f32,f32,f32)) -> ScoreHit {
        let (hitwindow_miss, hitwindow_100, hitwindow_300) = hit_windows;
        let diff = (time - self.time).abs();
        
        // make sure the cursor is in the radius
        let distance = ((self.pos.x - self.mouse_pos.x).powi(2) + (self.pos.y - self.mouse_pos.y).powi(2)).sqrt();

        if diff < hitwindow_300 {
            self.hit_time = time.max(0.0);
            self.hit = true;
            if distance > self.radius {
                ScoreHit::Miss
            } else {
                ScoreHit::X300
            }
        } else if diff < hitwindow_100 {
            self.hit_time = time.max(0.0);
            self.hit = true;
            if distance > self.radius {
                ScoreHit::Miss
            } else {
                ScoreHit::X100
            }
        } else if diff < hitwindow_miss { // too early, miss
            self.hit_time = time.max(0.0);
            self.missed = true;
            ScoreHit::Miss
        } else { // way too early, ignore
            ScoreHit::None
        }
    }
}



// slider
#[derive(Clone)]
pub struct StandardSlider {
    /// start pos
    pos: Vector2,
    /// end pos
    end_pos: Vector2,

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
    time_preempt:f32
}
impl StandardSlider {
    pub fn new(def:SliderDef, curve:Curve, ar:f32, cs:f32, color:Color, combo_num: u16) -> Self {
        let cs_scale = (1.0 - 0.7 * (cs - 5.0) / 5.0) / 2.0;
        let time_preempt = map_difficulty(ar, 1800.0, 1200.0, PREEMPT_MIN);
        let end_pos = POS_OFFSET + curve.position_at_length(curve.length());

        Self {
            curve,
            color,
            combo_num,
            time_preempt,
            base_depth: get_depth(def.time),
            radius: CIRCLE_RADIUS_BASE * cs_scale as f64,

            pos: POS_OFFSET + def.pos,
            end_pos,
            time: def.time, 
            hit_dots: Vec::new(),
            map_time: 0.0,

            hold_time: 0.0,
            release_time: 0.0,
            mouse_pos: Vector2::zero()
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
            list.push(timing_circle(self.pos, self.radius, self.time - self.map_time, self.time_preempt, self.base_depth));
        } else {
            // slider ball
            let pos = POS_OFFSET + self.curve.position_at_time(self.map_time);
            let distance = ((pos.x - self.mouse_pos.x).powi(2) + (pos.y - self.mouse_pos.y).powi(2)).sqrt();
            let mut c = Circle::new(
                self.color,
                self.base_depth - 1.0,
                pos,
                self.radius
            );
            c.border = Some(Border::new(
                if self.hold_time > self.release_time && distance <= self.radius {
                    Color::RED
                } else {
                    Color::WHITE
                },
                2.0
            ));
            list.push(Box::new(c));
        }

        // curves
        list.reserve(self.curve.path.len() * 2);
        for i in 0..self.curve.path.len() {
            let color = self.color;

            let line = self.curve.path[i];
            list.push(Box::new(Line::new(
                POS_OFFSET + line.p1,
                POS_OFFSET + line.p2,
                self.radius,
                self.base_depth,
                color
            )));
            // add a circle to smooth out the corners
            list.push(Box::new(Circle::new(
                color,
                self.base_depth,
                POS_OFFSET + line.p2,
                self.radius,
            )))
        }
        
        // start and end circles
        for pos in [self.pos, self.end_pos] {
            let mut c = Circle::new(
                Color::YELLOW,
                self.base_depth - 0.5, // should be above curves but below slider ball
                pos,
                self.radius
            );
            c.border = Some(Border {
                color: Color::BLACK.into(),
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
    fn get_preempt(&self) -> f32 {self.time_preempt}
    fn press(&mut self, time:f32) {self.hold_time = time}
    fn release(&mut self, time:f32) {self.release_time = time}
    fn mouse_move(&mut self, pos:Vector2) {self.mouse_pos = pos}

    fn get_points(&mut self, _time:f32, _:(f32,f32,f32)) -> ScoreHit {

        // self.hit_dots.push(SliderDot::new(time, self.speed));
        ScoreHit::Other(100, false)
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
    pub fn update(&mut self, _beatmap_time:f64) {

    }
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
    /// how many rotations is needed to pass this spinner
    rotations_required: u16,
    /// how many rotations have been completed?
    rotations_completed: u16,

    /// should we count mouse movements?
    holding: bool
}
impl StandardSpinner {
    pub fn new(time:f32, end_time:f32) -> Self {
        Self {
            time, 
            end_time,

            holding: false,
            rotation: 0.0,
            rotations_required: 0,
            rotations_completed: 0
        }
    }
}
impl HitObject for StandardSpinner {
    fn note_type(&self) -> NoteType {NoteType::Spinner}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self,_:f32) -> f32 {self.end_time}

    fn update(&mut self, _beatmap_time: f32) {}
    fn draw(&mut self, _args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        // if its time to start hitting the spinner
        // bg circle
        let mut bg = Circle::new(
            Color::YELLOW,
            -10.0,
            SPINNER_POSITION,
            SPINNER_RADIUS
        );
        bg.border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE));
        list.push(Box::new(bg));

        // draw another circle on top which increases in radius as the counter gets closer to the reqired
        let mut fg = Circle::new(
            Color::WHITE,
            -11.0,
            SPINNER_POSITION,
            SPINNER_RADIUS * (self.rotations_completed as f64 / self.rotations_required as f64)
        );
        fg.border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE));
        list.push(Box::new(fg));
            

        //TODO: draw a counter
    }

    fn reset(&mut self) {
        self.rotation = 0.0;
        self.rotations_completed = 0;
    }
}
impl StandardHitObject for StandardSpinner {
    fn get_preempt(&self) -> f32 {0.0}
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
        
    }
}


fn get_depth(time:f32) -> f64 {
    (NOTE_DEPTH + time as f64) / 1000.0
}
fn timing_circle(pos:Vector2, radius:f64, time_diff: f32, time_preempt:f32, depth: f64) -> Box<Circle> {
    let mut c = Circle::new(
        Color::TRANSPARENT_WHITE,
        depth - 100.0,
        pos,
        radius + (time_diff as f64 / time_preempt as f64) * HITWINDOW_CIRCLE_RADIUS
    );
    c.border = Some(Border::new(Color::WHITE, NOTE_BORDER_SIZE));
    Box::new(c)
}