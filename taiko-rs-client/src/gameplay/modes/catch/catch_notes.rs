use piston::RenderArgs;

use taiko_rs_common::types::ScoreHit;
use crate::Vector2;
use crate::gameplay::HitObject;
use crate::gameplay::NoteType;
use crate::render::{Circle, Color, Renderable, Border};
use super::HIT_Y;

const NOTE_BORDER_SIZE:f64 = 2.0;

pub trait CatchHitObject: HitObject {
    /// does this object count as a miss if it is not hit?
    fn causes_miss(&self) -> bool; //TODO: might change this to return an enum of "no", "yes". "yes_combo_only"
    fn get_points(&mut self) -> ScoreHit;
    fn speed(&self) -> f64;
    fn radius(&self) -> f64;
    fn x(&self) -> f64;
    fn y_at(&self, time:i64) -> f64 {
        HIT_Y 
        - (
            (self.time() as f64 - time as f64) 
            * self.speed() 
            - (self.radius() + NOTE_BORDER_SIZE / 2.0)
        )
    }
}

// normal note
#[derive(Clone, Copy)]
pub struct CatchFruit {
    pos: Vector2,
    time: u64, // ms
    hit: bool,
    missed: bool,
    speed: f64,
    radius: f64
}
impl CatchFruit {
    pub fn new(time:u64, speed:f64, radius:f64, x:f64) -> Self {
        Self {
            time, 
            speed,
            radius,
            hit: false,
            missed: false,
            pos: Vector2::new(x, 0.0),
        }
    }
}
impl HitObject for CatchFruit {
    fn note_type(&self) -> NoteType {NoteType::Note}
    fn time(&self) -> u64 {self.time}
    fn end_time(&self, _:f64) -> u64 {self.time}
    fn update(&mut self, beatmap_time: i64) {
        self.pos.y = self.y_at(beatmap_time);
    }
    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut renderables: Vec<Box<dyn Renderable>> = Vec::new();
        if self.pos.y + self.radius < 0.0 || self.pos.y - self.radius > args.window_size[1] as f64 || self.hit {return renderables}

        let mut note = Circle::new(
            Color::new(0.8, 0.0, 0.0, 1.0),
            -100.0,
            self.pos,
            self.radius
        );
        note.border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE));
        renderables.push(Box::new(note));

        renderables
    }

    fn reset(&mut self) {
        self.pos.y = 0.0;
        self.hit = false;
        self.missed = false;
    }
}
impl CatchHitObject for CatchFruit {
    fn x(&self) -> f64 {self.pos.x}
    fn speed(&self) -> f64 {self.speed}
    fn radius(&self) -> f64 {self.radius}
    fn causes_miss(&self) -> bool {true}
    fn get_points(&mut self) -> ScoreHit {
        self.hit = true;
        ScoreHit::X300
    }
}


// slider droplet
#[derive(Clone, Copy)]
pub struct CatchDroplet {
    pos: Vector2,
    time: u64, // ms
    hit: bool,
    missed: bool,
    speed: f64,
    radius: f64
}
impl CatchDroplet {
    pub fn new(time:u64, speed:f64, radius:f64, x:f64) -> Self {
        Self {
            time, 
            speed,
            radius,
            hit: false,
            missed: false,
            pos: Vector2::new(x, 0.0),
        }
    }
}
impl HitObject for CatchDroplet {
    fn note_type(&self) -> NoteType {NoteType::Note}
    fn time(&self) -> u64 {self.time}
    fn end_time(&self, _:f64) -> u64 {self.time}
    fn update(&mut self, beatmap_time: i64) {
        self.pos.y = self.y_at(beatmap_time);
    }
    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut renderables: Vec<Box<dyn Renderable>> = Vec::new();
        if self.pos.y + self.radius < 0.0 || self.pos.y - self.radius > args.window_size[1] as f64 || self.hit {return renderables}

        let mut note = Circle::new(
            Color::new(0.8, 0.0, 0.0, 1.0),
            -100.0,
            self.pos,
            self.radius
        );
        note.border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE));
        renderables.push(Box::new(note));

        renderables
    }

    fn reset(&mut self) {
        self.pos.y = 0.0;
        self.hit = false;
        self.missed = false;
    }
}
impl CatchHitObject for CatchDroplet {
    fn x(&self) -> f64 {self.pos.x}
    fn speed(&self) -> f64 {self.speed}
    fn radius(&self) -> f64 {self.radius}
    fn causes_miss(&self) -> bool {true}
    fn get_points(&mut self) -> ScoreHit {
        self.hit = true;
        ScoreHit::X100
    }
}
