use core::f32;

use ayyeve_piston_ui::render::Line;
use piston::RenderArgs;

use taiko_rs_common::types::ScoreHit;
use crate::Vector2;
use crate::WINDOW_SIZE;
use crate::gameplay::HitObject;
use crate::gameplay::NoteDef;
use crate::gameplay::NoteType;
use crate::gameplay::SliderDef;
use crate::gameplay::modes::FIELD_SIZE;
use crate::helpers::slider::Curve;
use crate::render::{Circle, Color, Renderable, Border};

const SPINNER_RADIUS:f64 = 200.0;
const SPINNER_POSITION:Vector2 = Vector2::new(WINDOW_SIZE.x / 2.0, WINDOW_SIZE.y / 2.0);
const SLIDER_DOT_RADIUS:f64 = 8.0;
const NOTE_BORDER_SIZE:f64 = 2.0;

const CIRCLE_RADIUS_BASE: f64 = 40.0;
const CIRCLE_TIMING_TIME: f32 = 1000.0;


pub const POS_OFFSET:Vector2 = Vector2::new((WINDOW_SIZE.x - FIELD_SIZE.x) / 2.0, (WINDOW_SIZE.y - FIELD_SIZE.y) / 2.0);


pub trait StandardHitObject: HitObject {
    /// does this object count as a miss if it is not hit?
    fn causes_miss(&self) -> bool; //TODO: might change this to return an enum of "no", "yes". "yes_combo_only"
    fn get_points(&mut self, time:f32, hit_windows:(f32,f32,f32)) -> ScoreHit;

    fn press(&mut self, time:f32);
    fn release(&mut self, time:f32) {}
    fn mouse_move(&mut self, pos:Vector2);
}


// note
#[derive(Clone, Copy)]
pub struct StandardNote {
    pos: Vector2,
    time: f32, // ms
    hit_time: f32,
    hit: bool,
    missed: bool,

    color: Color, 
    combo_num: u16,

    map_time: f32,
    mouse_pos: Vector2,
    radius: f64
}
impl StandardNote {
    pub fn new(def:NoteDef, ar:f32, cs:f32, color:Color, combo_num:u16) -> Self {
        Self {
            pos: POS_OFFSET + def.pos,
            time: def.time, 
            color,
            combo_num,
            
            hit: false,
            hit_time: 0.0,
            missed: false,

            map_time: 0.0,
            mouse_pos: Vector2::zero(),

            radius: CIRCLE_RADIUS_BASE
        }
    }
}
impl HitObject for StandardNote {
    fn note_type(&self) -> NoteType {NoteType::Note}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self, hw_miss:f32) -> f32 {self.time + hw_miss}
    fn update(&mut self, beatmap_time: f32) {
        self.map_time = beatmap_time;
    }
    fn draw(&mut self, _args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        if self.time - self.map_time > CIRCLE_TIMING_TIME || self.time - self.map_time < 0.0 {return}

        // timing circle
        list.push(Box::new(timing_circle(self.pos, self.radius, self.time - self.map_time)));

        // note
        let mut note = Circle::new(
            self.color,
            -100.0,
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

    fn get_points(&mut self, time:f32, hit_windows:(f32,f32,f32)) -> ScoreHit {
        let (hitwindow_miss, hitwindow_100, hitwindow_300) = hit_windows;
        let diff = (time - self.time).abs();
        
        // make sure the cursor is in the radius
        let distance = ((self.pos.x - self.mouse_pos.x).powi(2) + (self.pos.y - self.mouse_pos.y).powi(2)).sqrt();
        println!("distance: {}, r: {}", distance, self.radius);
        if distance > self.radius {
            return ScoreHit::Miss
        }

        if diff < hitwindow_300 {
            self.hit_time = time.max(0.0);
            self.hit = true;
            ScoreHit::X300
        } else if diff < hitwindow_100 {
            self.hit_time = time.max(0.0);
            self.hit = true;
            ScoreHit::X100
        } else if diff < hitwindow_miss { // too early, miss
            self.hit_time = time.max(0.0);
            self.missed = true;
            ScoreHit::Miss
        } else { // way too early, ignore
            ScoreHit::None
        }
    }

    fn press(&mut self, time:f32) {
        self.hit_time = time;
    }
    
    fn mouse_move(&mut self, pos:Vector2) {
        self.mouse_pos = pos;
    }
}



// slider
#[derive(Clone)]
pub struct StandardSlider {
    /// start pos
    pos: Vector2,

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

    map_time: f32,
}
impl StandardSlider {
    pub fn new(def:SliderDef, curve:Curve, color:Color, combo_num: u16) -> Self {
        Self {
            curve,
            color,
            combo_num,
            radius: CIRCLE_RADIUS_BASE,

            time: def.time, 
            pos: POS_OFFSET + def.pos,
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
    fn end_time(&self,_:f32) -> f32 {self.curve.end_time} //TODO
    fn update(&mut self, beatmap_time: f32) {
        self.map_time = beatmap_time;
    }
    fn draw(&mut self, _args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        if self.time - self.map_time > CIRCLE_TIMING_TIME || self.curve.end_time < self.map_time {return}

        if self.time - self.map_time > 0.0 {
            // timing circle
            list.push(Box::new(timing_circle(self.pos, self.radius, self.time - self.map_time)));
        } else {
            let pos = POS_OFFSET + self.curve.position_at_time(self.map_time);
            let distance = ((pos.x - self.mouse_pos.x).powi(2) + (pos.y - self.mouse_pos.y).powi(2)).sqrt();
            // slider ball
            let mut c = Circle::new(
                self.color,
                -101.0,
                pos,
                self.radius
            );
            c.border = Some(Border::new(
                if self.hold_time > self.release_time && distance <= self.radius {
                    Color::RED
                } else {
                    println!("slider distance: {}", distance);
                    Color::WHITE
                },
                2.0
            ));
            list.push(Box::new(c));
        }

        // curves
        for i in 0..self.curve.path.len() {
            let line = self.curve.path[i];
            list.push(Box::new(Line::new(
                POS_OFFSET + line.p1,
                POS_OFFSET + line.p2,
                self.radius,
                -100.0,
                self.color
            )))
        }
        // end

        // start and end circles
        for pos in [self.pos, POS_OFFSET + self.curve.position_at_time(self.curve.end_time-1.0)] {
            let mut c = Circle::new(
                Color::YELLOW,
                -100.5,
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

    fn get_points(&mut self, _time:f32, _:(f32,f32,f32)) -> ScoreHit {

        // self.hit_dots.push(SliderDot::new(time, self.speed));
        ScoreHit::Other(100, false)
    }


    fn press(&mut self, time:f32) {
        self.hold_time = time;
    }
    fn release(&mut self, time:f32) {
        self.release_time = time;
    }
    fn mouse_move(&mut self, pos:Vector2) {
        self.mouse_pos = pos;
    }
}

/// helper struct for drawing hit slider points
#[derive(Clone, Copy)]
struct SliderDot {
    time: f64,
    speed: f64,
    pos: Vector2,
    pub done: bool
}
impl SliderDot {
    pub fn new(time:f64, speed:f64) -> SliderDot {
        SliderDot {
            time,
            speed,
            pos: Vector2::zero(),
            done: false
        }
    }
    pub fn update(&mut self, beatmap_time:f64) {
        // let y = -((beatmap_time as f64 - self.time as f64)*20.0).ln()*20.0 + 1.0;
        // self.pos = HIT_POSITION + Vector2::new((self.time as f64 - beatmap_time as f64) * self.speed, y);
        
        // if !self.done && self.pos.x - SLIDER_DOT_RADIUS <= 0.0 {
        //     self.done = true;
        // }
    }
    pub fn draw(&self) -> Vec<Box<dyn Renderable>> {

        let mut c = Circle::new(
            Color::YELLOW,
            -100.0,
            self.pos,
            SLIDER_DOT_RADIUS
        );
        c.border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE/2.0));

        vec![]
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

    fn update(&mut self, beatmap_time: f32) {
        // self.pos = HIT_POSITION + Vector2::new((self.time as f64 - beatmap_time as f64) * self.speed, 0.0);
        // if beatmap_time > self.end_time as i64 {self.complete = true}
    }
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



fn timing_circle(pos:Vector2, radius:f64, time_diff: f32) -> Circle {
    let mut c = Circle::new(
        Color::TRANSPARENT_WHITE,
        -110.0,
        pos,
        radius + (time_diff as f64 / CIRCLE_TIMING_TIME as f64) * 20.0
    );
    c.border = Some(Border::new(Color::WHITE, NOTE_BORDER_SIZE));
    c
}