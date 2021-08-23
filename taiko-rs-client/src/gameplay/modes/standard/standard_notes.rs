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
use crate::helpers::slider::Curve;
use crate::helpers::slider::get_curve;
use crate::render::{Circle, Color, Renderable, Border};

const SPINNER_RADIUS:f64 = 200.0;
const SPINNER_POSITION:Vector2 = Vector2::new(WINDOW_SIZE.x / 2.0, WINDOW_SIZE.y / 2.0);
const SLIDER_DOT_RADIUS:f64 = 8.0;
const NOTE_BORDER_SIZE:f64 = 2.0;


const CIRCLE_RADIUS_BASE: f64 = 30.0;


pub trait StandardHitObject: HitObject {
    /// does this object count as a miss if it is not hit?
    fn causes_miss(&self) -> bool; //TODO: might change this to return an enum of "no", "yes". "yes_combo_only"
    fn get_points(&mut self, time:f32, hit_windows:(f32,f32,f32)) -> ScoreHit;
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
    combo_num: u16
}
impl StandardNote {
    pub fn new(def:NoteDef, ar:f32, cs:f32, color:Color, combo_num:u16) -> Self {
        Self {
            pos: def.pos,
            time: def.time, 
            color,
            combo_num,
            
            hit: false,
            hit_time: 0.0,
            missed: false,
        }
    }
}
impl HitObject for StandardNote {
    fn note_type(&self) -> NoteType {NoteType::Note}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self, hw_miss:f32) -> f32 {self.time + hw_miss}
    fn update(&mut self, beatmap_time: f32) {}
    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut renderables: Vec<Box<dyn Renderable>> = Vec::new();


        let mut note = Circle::new(
            self.color,
            -100.0,
            self.pos,
            CIRCLE_RADIUS_BASE
        );
        note.border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE));
        renderables.push(Box::new(note));

        renderables
    }

    fn reset(&mut self) {
        self.pos = Vector2::zero();
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
}


// slider
#[derive(Clone)]
pub struct StandardSlider {
    pos: Vector2,
    hit_dots: Vec<SliderDot>,

    time: f32, // ms
    // end_time: u64, // ms

    curve: Curve,

    color: Color,
    combo_num: u16
}
impl StandardSlider {
    pub fn new(def:SliderDef, curve:Curve, color:Color, combo_num: u16) -> Self {
        Self {
            curve,
            color,
            combo_num,

            time: def.time, 
            pos: def.pos,
            hit_dots: Vec::new()
        }
    }
}
impl HitObject for StandardSlider {
    fn note_type(&self) -> NoteType {NoteType::Slider}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self,_:f32) -> f32 {self.time} //TODO
    fn update(&mut self, beatmap_time: f32) {
        // self.pos.x = HIT_POSITION.x + (self.time as f64 - beatmap_time as f64) * self.speed;
        // self.end_x = HIT_POSITION.x + (self.end_time(0.0) as f64 - beatmap_time as f64) * self.speed;

        // // draw hit dots
        // for dot in self.hit_dots.as_mut_slice() {
        //     if dot.done {continue;}
        //     dot.update(beatmap_time as f64);
        // }
    }
    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut renderables: Vec<Box<dyn Renderable>> = Vec::new();

        // middle

        for i in 0..self.curve.path.len() {
            let line = self.curve.path[i];
            renderables.push(Box::new(Line::new(
                line.p1,
                line.p2,
                CIRCLE_RADIUS_BASE,
                -100.0,
                self.color
            )))
        }

        // renderables.push(Box::new(Rectangle::new(
        //     Color::YELLOW,
        //     self.time as f64 + 1.0,
        //     self.pos,
        //     Vector2::new(self.end_x - self.pos.x , self.radius * 2.0),
        //     Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE))
        // )));

        // // start circle
        // let mut start_c = Circle::new(
        //     Color::YELLOW,
        //     self.time as f64,
        //     self.pos + Vector2::new(0.0, self.radius),
        //     self.radius
        // );
        // start_c.border = Some(Border {
        //     color: Color::BLACK.into(),
        //     radius: NOTE_BORDER_SIZE
        // });
        // renderables.push(Box::new(start_c));
        
        // // end circle
        // let mut end_c = Circle::new(
        //     Color::YELLOW,
        //     self.time as f64,
        //     Vector2::new(self.end_x, self.pos.y + self.radius),
        //     self.radius
        // );
        // end_c.border = Some(Border {
        //     color: Color::BLACK.into(),
        //     radius: NOTE_BORDER_SIZE
        // });
        // renderables.push(Box::new(end_c));

        // draw hit dots
        // for dot in self.hit_dots.as_slice() {
        //     if dot.done {continue}
        //     renderables.extend(dot.draw());
        // }
        
        renderables
    }

    fn reset(&mut self) {
        self.hit_dots.clear();
    }
}
impl StandardHitObject for StandardSlider {

    fn causes_miss(&self) -> bool {false}

    fn get_points(&mut self, time:f32, _:(f32,f32,f32)) -> ScoreHit {

        // self.hit_dots.push(SliderDot::new(time, self.speed));
        ScoreHit::Other(100, false)
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
    pos: Vector2, // the note in the bar, not the spinner itself

    time: f32, // ms
    end_time: f32, // ms

    /// current angle of the spinner
    rotation: f64,
    /// how many rotations is needed to pass this spinner
    rotations_required: u16,
    /// how many rotations have been completed?
    rotations_completed: u16
}
impl StandardSpinner {
    pub fn new(time:f32, end_time:f32) -> Self {
        Self {
            time, 
            end_time,

            rotation: 0.0,
            rotations_required: 0,
            rotations_completed: 0,
            pos: SPINNER_POSITION,
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
    fn draw(&mut self, _args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut renderables: Vec<Box<dyn Renderable>> = Vec::new();

        // if its time to start hitting the spinner
        // bg circle
        let mut bg = Circle::new(
            Color::YELLOW,
            -10.0,
            SPINNER_POSITION,
            SPINNER_RADIUS
        );
        bg.border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE));
        renderables.push(Box::new(bg));

        // draw another circle on top which increases in radius as the counter gets closer to the reqired
        let mut fg = Circle::new(
            Color::WHITE,
            -11.0,
            SPINNER_POSITION,
            SPINNER_RADIUS * (self.rotations_completed as f64 / self.rotations_required as f64)
        );
        fg.border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE));
        renderables.push(Box::new(fg));
            

        //TODO: draw a counter
        
        renderables
    }

    fn reset(&mut self) {
        self.rotation = 0.0;
        self.rotations_completed = 0;
    }
}
impl StandardHitObject for StandardSpinner {
    fn causes_miss(&self) -> bool {self.rotations_completed < self.rotations_required} // if the spinner wasnt completed in time, cause a miss

    fn get_points(&mut self, time:f32, _:(f32,f32,f32)) -> ScoreHit {
        ScoreHit::Other(100, false)
    }
}