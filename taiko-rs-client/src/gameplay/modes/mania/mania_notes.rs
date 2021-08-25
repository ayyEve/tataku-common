use piston::RenderArgs;

use crate::Vector2;
use crate::gameplay::{HitObject, defs::NoteType};
use crate::render::{Color, Rectangle, Renderable, Border};

use super::{NOTE_BORDER_SIZE, NOTE_SIZE, COLUMN_WIDTH, HIT_Y};


pub trait ManiaHitObject: HitObject {
    fn hit(&mut self, time:f32);
    fn release(&mut self, _time:f32) {}
    fn miss(&mut self, time:f32);
    fn was_hit(&self) -> bool {false}

    fn y_at(&self, time:f32) -> f64;

    fn set_sv(&mut self, sv:f32);
}

// note
#[derive(Clone, Copy)]
pub struct ManiaNote {
    pos: Vector2,
    time: f32, // ms
    hit_time: f32,
    hit: bool,
    missed: bool,
    speed: f32
}
impl ManiaNote {
    pub fn new(time:f32, x:f64) -> Self {
        Self {
            time, 
            speed: 1.0,

            hit_time: 0.0,
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
    fn time(&self) -> f32 {self.time}
    fn end_time(&self, hw_miss:f32) -> f32 {self.time + hw_miss}

    fn update(&mut self, beatmap_time: f32) {
        // let y = 
        //     if self.hit {-((beatmap_time as f64 - self.hit_time as f64)*20.0).ln()*20.0 + 1.0} 
        //     else if self.missed {GRAVITY_SCALING * 9.81 * ((beatmap_time as f64 - self.hit_time as f64)/1000.0).powi(2)} 
        //     else {0.0};
        
        self.pos.y = self.y_at(beatmap_time); //HIT_Y - (self.time as f64 - beatmap_time as f64) * self.speed;
    }
    fn draw(&mut self, args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        if self.pos.y + NOTE_SIZE.y < 0.0 || self.pos.y > args.window_size[1] as f64 {return}

        if self.hit {return}

        let note = Rectangle::new(
            self.get_color(),
            -100.0,
            self.pos,
            NOTE_SIZE,
            Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE))
        );
        list.push(Box::new(note));
    }

    fn reset(&mut self) {
        self.pos.y = 0.0;
        self.hit = false;
        self.missed = false;
        self.hit_time = 0.0;
    }
}
impl ManiaHitObject for ManiaNote {
    fn hit(&mut self, time:f32) {
        self.hit = true;
        self.hit_time = time;
    }
    fn miss(&mut self, time:f32) {
        self.missed = true;
        self.hit_time = time;
    }

    fn y_at(&self, time:f32) -> f64 {
        HIT_Y - ((self.time - time) * self.speed) as f64
    }

    fn set_sv(&mut self, sv:f32) {
        self.speed = sv;
    }
}

// slider
#[derive(Clone)]
pub struct ManiaHold {
    pos: Vector2,
    time: f32, // ms
    end_time: f32, // ms

    /// when the user started holding
    hold_starts: Vec<f32>,
    hold_ends: Vec<f32>,
    holding: bool,

    speed: f32,
    //TODO: figure out how to pre-calc this
    end_y: f64
}
impl ManiaHold {
    pub fn new(time:f32, end_time:f32, x:f64) -> Self {
        Self {
            time, 
            end_time,
            speed: 1.0,
            holding:false,

            pos: Vector2::new(x, 0.0),
            hold_starts: Vec::new(),
            hold_ends: Vec::new(),
            end_y: 0.0,
        }
    }
}
impl HitObject for ManiaHold {
    fn note_type(&self) -> NoteType {NoteType::Hold}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self,hw_miss:f32) -> f32 {self.end_time + hw_miss}

    fn update(&mut self, beatmap_time: f32) {
        // self.pos.x = HIT_POSITION.x + (self.time as f64 - beatmap_time as f64) * self.speed;
        self.end_y = HIT_Y - ((self.end_time - beatmap_time) * self.speed) as f64;
        self.pos.y = HIT_Y - ((self.time - beatmap_time) * self.speed) as f64;
    }
    fn draw(&mut self, args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        if self.pos.y < 0.0 || self.end_y > args.window_size[1] as f64 {return}

        // start
        if self.pos.y < HIT_Y {
            list.push(Box::new(Rectangle::new(
                Color::YELLOW,
                -100.1,
                self.pos,
                NOTE_SIZE,
                Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE))
            )));
        }

        // end
        if self.end_y < HIT_Y {
            list.push(Box::new(Rectangle::new(
                Color::YELLOW,
                -100.1,
                Vector2::new(self.pos.x, self.end_y),
                NOTE_SIZE,
                Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE))
            )));
        }

        // draw hold fragments
        // for i in 0..self.hold_ends.len() {
        //     let start = self.hold_starts[i];
        //     let end = self.hold_ends[i];
        //     let y = HIT_Y - (end - start) * self.speed;

        //     list.push(Box::new(Rectangle::new(
        //         Color::YELLOW,
        //         -100.0,
        //         Vector2::new(self.pos.x, y),
        //         Vector2::new(COLUMN_WIDTH, self.end_y - y),
        //         Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE))
        //     )));
        // }

        // middle
        if self.end_y < HIT_Y {
            let y = if self.holding {HIT_Y} else {self.pos.y};
            list.push(Box::new(Rectangle::new(
                Color::YELLOW,
                -100.0,
                Vector2::new(self.pos.x, y),
                Vector2::new(COLUMN_WIDTH, self.end_y - y),
                Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE))
            )));
        }
    }

    fn reset(&mut self) {
        self.pos.y = 0.0;
        self.hold_starts.clear();
        self.hold_ends.clear();
    }
}
impl ManiaHitObject for ManiaHold {
    fn was_hit(&self) -> bool {
        self.hold_starts.len() > 0  
    }

    // key pressed
    fn hit(&mut self, time:f32) {
        self.hold_starts.push(time);
        self.holding = true;
    }
    fn release(&mut self, time:f32) {
        self.hold_ends.push(time);
        self.holding = false;
    }

    //
    fn miss(&mut self, _time:f32) {}

    fn y_at(&self, time:f32) -> f64 {
        HIT_Y - ((self.time - time) * self.speed) as f64
    }

    fn set_sv(&mut self, sv:f32) {
        self.speed = sv;
    }
}