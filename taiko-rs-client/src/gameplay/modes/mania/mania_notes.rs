use piston::RenderArgs;

use taiko_rs_common::types::KeyPress;
use taiko_rs_common::types::ScoreHit;
use crate::Vector2;
use crate::gameplay::HitObject;
use crate::gameplay::NoteType;
use crate::gameplay::modes::taiko::HitType;
use crate::render::{Color, Rectangle, Renderable, Border};

use super::COLUMN_WIDTH;
use super::HIT_Y;
use super::{NOTE_BORDER_SIZE, NOTE_SIZE};


const GRAVITY_SCALING:f64 = 400.0;


// note
#[derive(Clone, Copy)]
pub struct ManiaNote {
    pos: Vector2,
    time: u64, // ms
    hit_time: u64,
    hit: bool,
    missed: bool,
    speed: f64
}
impl ManiaNote {
    pub fn new(time:u64, x:f64, speed:f64) -> Self {
        Self {
            time, 
            speed,

            hit_time: 0,
            hit: false,
            missed: false,
            pos: Vector2::new(x, 0.0),
        }
    }

    fn get_color(&mut self) -> Color {
        Color::WHITE
    }
}
impl HitObject for ManiaNote {
    fn note_type(&self) -> NoteType {NoteType::Note}
    fn set_sv(&mut self, sv:f64) {self.speed = sv}
    fn time(&self) -> u64 {self.time}
    fn end_time(&self, hw_miss:f64) -> u64 {self.time + hw_miss as u64}
    fn causes_miss(&self) -> bool {true}
    fn x_at(&self, time:i64) -> f64 {(self.time as f64 - time as f64) * self.speed}

    fn get_points(&mut self, _hit_type:HitType, time:f64, hit_windows:(f64,f64,f64)) -> ScoreHit {
        let (hitwindow_miss, hitwindow_100, hitwindow_300) = hit_windows;
        let diff = (time - self.time as f64).abs();

        if diff < hitwindow_300 {
            self.hit_time = time.max(0.0) as u64;
            self.hit = true;
            ScoreHit::X300
        } else if diff < hitwindow_100 {
            self.hit_time = time.max(0.0) as u64;
            self.hit = true;
            ScoreHit::X100
        } else if diff < hitwindow_miss { // too early, miss
            self.hit_time = time.max(0.0) as u64;
            self.missed = true;
            ScoreHit::Miss
        } else { // way too early, ignore
            ScoreHit::None
        }
    }
    fn update(&mut self, beatmap_time: i64) {
        // let y = 
        //     if self.hit {-((beatmap_time as f64 - self.hit_time as f64)*20.0).ln()*20.0 + 1.0} 
        //     else if self.missed {GRAVITY_SCALING * 9.81 * ((beatmap_time as f64 - self.hit_time as f64)/1000.0).powi(2)} 
        //     else {0.0};
        
        self.pos.y = HIT_Y - (self.time as f64 - beatmap_time as f64) * self.speed;
    }
    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut renderables: Vec<Box<dyn Renderable>> = Vec::new();

        // if self.pos.x + NOTE_RADIUS < 0.0 || self.pos.x - NOTE_RADIUS > args.window_size[0] as f64 {return renderables}
        if self.hit {
            return renderables;
        }

        let note = Rectangle::new(
            self.get_color(),
            -100.0,
            self.pos,
            NOTE_SIZE,
            Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE))
        );
        renderables.push(Box::new(note));

        renderables
    }

    fn reset(&mut self) {
        self.pos.y = 0.0;
        self.hit = false;
        self.missed = false;
        self.hit_time = 0;
    }
}

// slider
#[derive(Clone)]
pub struct ManiaHold {
    pos: Vector2,
    time: u64, // ms
    end_time: u64, // ms

    /// when the user started holding
    hold_start: f64,

    speed: f64,
    //TODO: figure out how to pre-calc this
    end_y: f64
}
impl ManiaHold {
    pub fn new(time:u64, end_time:u64, x:f64, speed:f64) -> Self {
        Self {
            time, 
            end_time,
            speed,

            pos: Vector2::new(x, 0.0),
            hold_start: 0.0,
            end_y: 0.0,
        }
    }
}
impl HitObject for ManiaHold {
    fn note_type(&self) -> NoteType {NoteType::Hold}
    fn set_sv(&mut self, sv:f64) {self.speed = sv}
    fn time(&self) -> u64 {self.time}
    fn end_time(&self,_:f64) -> u64 {self.end_time}
    fn causes_miss(&self) -> bool {true}
    fn x_at(&self, time:i64) -> f64 {(self.time as f64 - time as f64) * self.speed}

    fn get_points(&mut self, _hit_type:HitType, time:f64, _:(f64,f64,f64)) -> ScoreHit {
        println!("slider hit");
        // too soon or too late
        if time < self.time as f64 || time > self.end_time as f64 {return ScoreHit::None}
        ScoreHit::Other(100, true)
    }

    fn update(&mut self, beatmap_time: i64) {
        // self.pos.x = HIT_POSITION.x + (self.time as f64 - beatmap_time as f64) * self.speed;
        self.end_y = HIT_Y - (self.end_time as f64 - beatmap_time as f64) * self.speed;
        self.pos.y = HIT_Y - (self.time as f64 - beatmap_time as f64) * self.speed;
    }
    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut renderables: Vec<Box<dyn Renderable>> = Vec::new();
        // if self.end_y + NOTE_RADIUS < 0.0 || self.pos.y - NOTE_RADIUS > args.window_size[1] as f64 {return renderables}


        // start
        let note = Rectangle::new(
            Color::YELLOW,
            -100.1,
            self.pos,
            NOTE_SIZE,
            Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE))
        );
        renderables.push(Box::new(note));

        // middle
        renderables.push(Box::new(Rectangle::new(
            Color::YELLOW,
            -100.0,
            self.pos,
            Vector2::new(COLUMN_WIDTH, self.end_y - self.pos.y),
            Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE))
        )));


        // end
        let note = Rectangle::new(
            Color::YELLOW,
            -100.1,
            self.pos + Vector2::new(0.0, self.end_y - self.pos.y),
            NOTE_SIZE,
            Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE))
        );
        renderables.push(Box::new(note));

        // start circle
        // let mut start_c = Circle::new(
        //     Color::YELLOW,
        //     self.time as f64,
        //     self.pos + Vector2::new(0.0, NOTE_RADIUS),
        //     NOTE_RADIUS
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
        
        renderables
    }

    fn reset(&mut self) {
        self.pos.y = 0.0;
        self.hold_start = 0.0;
    }
}

