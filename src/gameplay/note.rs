use std::sync::{Arc, Mutex};

use cgmath::Vector2;
use piston::RenderArgs;

use crate::{HIT_POSITION, NOTE_RADIUS, gameplay::{Beatmap, ScoreHit, BAR_COLOR}, render::{Circle, Color, HalfCircle, Rectangle, Renderable, Border}};

const SLIDER_DOT_RADIUS:f64 = 8.0;
const SPINNER_RADIUS: f64 = 200.0;
const SPINNER_POSITION: Vector2<f64> = Vector2::new(100.0, 0.0); // + hit position
const FINISHER_LENIENCY:u64 = 20; // ms
const NOTE_BORDER_SIZE:f64 = 2.0;

// hitobject trait, implemented by anything that should be hit
pub trait HitObject {
    fn note_type(&self) -> NoteType;
    fn is_kat(&self) -> bool {false}// needed for diff calc :/
    fn set_sv(&mut self, sv:f64);
    fn set_od(&mut self, _od:f64) {}
    /// does this hit object play a finisher sound when hit?
    fn finisher_sound(&self) -> bool {false}

    /// time in ms of this hit object
    fn time(&self) -> u64;
    /// when should the hitobject be considered "finished"
    fn end_time(&self) -> u64;
    /// does this object count as a miss if it is not hit?
    fn causes_miss(&self) -> bool; //TODO: might change this to return an enum of "no", "yes". "yes_combo_only"
    
    fn get_points(&mut self, hit_type:HitType, time:f64) -> ScoreHit; // if negative, counts as a miss
    fn check_finisher(&mut self, _hit_type:HitType, _time:f64) -> ScoreHit {ScoreHit::None}

    fn update(&mut self, beatmap_time: i64);
    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>>;

    /// set this object back to defaults
    fn reset(&mut self);
}

// note
#[derive(Clone)]
pub struct Note {
    pos: Vector2<f64>,
    pub time: u64, // ms
    hit_time: u64,
    pub hit_type: HitType,
    pub finisher:bool,
    pub beatmap: Arc<Mutex<Beatmap>>,
    pub hit: bool,
    pub speed:f64,

    od:f64,


    hitwindow_50:f64,
    hitwindow_100:f64,
    hitwindow_300:f64
}
impl Note {
    pub fn new(beatmap:Arc<Mutex<Beatmap>>, time:u64, hit_type:HitType, finisher:bool, speed:f64) -> Note {
        Note {
            beatmap,
            time, 
            hit_time: 0,
            hit_type, 
            finisher,
            speed,
            hit: false,
            pos: Vector2::new(0.0,0.0),

            // set later
            od: 1.0,
            hitwindow_50: 0.0,
            hitwindow_100: 0.0,
            hitwindow_300: 0.0
        }
    }

    fn get_color(&mut self) -> Color {
        match self.hit_type {
            HitType::Don => {
                return [1.0,0.0,0.0,1.0].into();
            }
            HitType::Kat => {
                return [0.0,0.0,1.0,1.0].into();
            }
        }
    }
}
impl HitObject for Note {
    fn set_sv(&mut self, sv:f64) {self.speed = sv}
    fn set_od(&mut self, od:f64) {
        self.od = od;

        self.hitwindow_50 = map_difficulty_range(od, 135.0, 95.0, 70.0);
        self.hitwindow_100 = map_difficulty_range(od, 120.0, 80.0, 50.0);
        self.hitwindow_300 = map_difficulty_range(od, 50.0, 35.0, 20.0);
    }
    fn note_type(&self) -> NoteType {NoteType::Note}
    fn is_kat(&self) -> bool {self.hit_type == HitType::Kat}
    fn finisher_sound(&self) -> bool {self.finisher}
    fn time(&self) -> u64 {self.time}
    fn end_time(&self) -> u64 {self.time + 100}
    fn causes_miss(&self) -> bool {true}
    fn get_points(&mut self, hit_type:HitType, time:f64) -> ScoreHit {
        let diff = (time - self.time as f64).abs();
        // println!("hit: {},{},{} : {}, {}", self.hitwindow_300, self.hitwindow_100, self.hitwindow_50, diff, self.od);

        if diff < self.hitwindow_300 {
            if hit_type != self.hit_type {
                ScoreHit::Miss
            } else {
                self.hit = true;
                self.hit_time = time.max(0.0) as u64;
                ScoreHit::X300
            }
        } else if diff < self.hitwindow_100 {
            if hit_type != self.hit_type {
                ScoreHit::Miss
            } else {
                self.hit = true;
                self.hit_time = time.max(0.0) as u64;
                ScoreHit::X100
            }
        } else if diff < self.hitwindow_50 { // too early, miss
            ScoreHit::Miss
        } else { // way too early, ignore
            ScoreHit::None
        }
    }

    fn check_finisher(&mut self, hit_type:HitType, time:f64) -> ScoreHit {
        if self.finisher && hit_type == self.hit_type && (time - self.hit_time as f64) < FINISHER_LENIENCY as f64 {
            ScoreHit::X300
        } else {
            ScoreHit::None
        }
    }

    fn update(&mut self, beatmap_time: i64) {

        let mut y = 0.0;
        if self.hit {
            y = -((beatmap_time as f64 - self.time as f64)*20.0).ln()*20.0 + 1.0;
        }
        
        self.pos = HIT_POSITION + Vector2::new((self.time as f64 - beatmap_time as f64) * self.speed, y);
    }
    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut renderables: Vec<Box<dyn Renderable>> = Vec::new();

        if self.pos.x + NOTE_RADIUS < 0.0 || self.pos.x - NOTE_RADIUS > args.window_size[0] as f64 {return renderables}

        let mut note = Circle::new(
            self.get_color(),
            self.time as f64,
            self.pos,
            if self.finisher {NOTE_RADIUS*1.6666} else {NOTE_RADIUS}
        );
        note.border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE));
        renderables.push(Box::new(note));

        renderables
    }

    fn reset(&mut self) {
        self.pos.x = 0.0;
        self.hit = false;
    }
}

// slider
pub struct Slider {
    pos: Vector2<f64>,
    hit_dots:Vec<SliderDot>, // list of times the slider was hit at

    pub time: u64, // ms
    pub end_time: u64, // ms
    pub finisher: bool,
    pub beatmap: Arc<Mutex<Beatmap>>,
    pub speed: f64,
    radius: f64,
    //TODO: figure out how to calc this
    end_x:f64
}
impl Slider {
    pub fn new(beatmap:Arc<Mutex<Beatmap>>, time:u64, end_time:u64, finisher:bool, speed:f64) -> Slider {
        let radius = if finisher {NOTE_RADIUS*1.6666} else {NOTE_RADIUS};

        Slider {
            beatmap,
            time, 
            end_time,
            finisher,
            speed,
            radius,

            pos: Vector2::new(0.0,HIT_POSITION.y - radius),
            end_x: 0.0,
            hit_dots: Vec::new()
        }
    }
}
impl HitObject for Slider {
    fn set_sv(&mut self, sv:f64) {self.speed = sv;}
    fn note_type(&self) -> NoteType {NoteType::Slider}
    fn time(&self) -> u64 {self.time}
    fn end_time(&self) -> u64 {self.end_time}
    fn causes_miss(&self) -> bool {false}
    fn get_points(&mut self, _hit_type:HitType, time:f64) -> ScoreHit {
        // too soon or too late
        if time < self.time as f64 || time > self.end_time as f64 {return ScoreHit::None}

        self.hit_dots.push(SliderDot::new(time, self.speed));
        ScoreHit::Other(100, false)
    }

    fn update(&mut self, beatmap_time: i64) {
        self.pos.x = HIT_POSITION.x + (self.time as f64 - beatmap_time as f64) * self.speed;
        self.end_x = HIT_POSITION.x + (self.end_time() as f64 - beatmap_time as f64) * self.speed;

        // draw hit dots
        for dot in self.hit_dots.as_mut_slice() {
            if dot.done {continue;}
            dot.update(beatmap_time as f64);
        }
    }
    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut renderables: Vec<Box<dyn Renderable>> = Vec::new();
        if self.end_x + NOTE_RADIUS < 0.0 || self.pos.x - NOTE_RADIUS > args.window_size[0] as f64 {return renderables}

        // middle
        renderables.push(Box::new(Rectangle::new(
            Color::YELLOW,
            self.time as f64 + 1.0,
            self.pos,
            Vector2::new(self.end_x - self.pos.x , self.radius * 2.0),
            Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE))
        )));

        // start circle
        let mut start_c = Circle::new(
            Color::YELLOW,
            self.time as f64,
            self.pos + Vector2::new(0.0, self.radius),
            self.radius
        );
        start_c.border = Some(Border {
            color: Color::BLACK.into(),
            radius: NOTE_BORDER_SIZE
        });
        renderables.push(Box::new(start_c));
        
        // end circle
        let mut end_c = Circle::new(
            Color::YELLOW,
            self.time as f64,
            Vector2::new(self.end_x, self.pos.y + self.radius),
            self.radius
        );
        end_c.border = Some(Border {
            color: Color::BLACK.into(),
            radius: NOTE_BORDER_SIZE
        });
        renderables.push(Box::new(end_c));

        // draw hit dots
        for dot in self.hit_dots.as_slice() {
            if dot.done {continue}
            renderables.extend(dot.draw());
        }
        
        renderables
    }

    fn reset(&mut self) {
        self.hit_dots.clear();
        self.pos.x = 0.0;
        self.end_x = 0.0;
    }
}
/// helper struct for drawing hit slider points
struct SliderDot {
    time: f64,
    speed: f64,
    pos: Vector2<f64>,
    pub done: bool
}
impl SliderDot {
    pub fn new(time:f64, speed:f64) -> SliderDot {
        SliderDot {
            time,
            speed,
            pos: Vector2::new(0.0, 0.0),
            done: false
        }
    }
    pub fn update(&mut self, beatmap_time:f64) {
        let y = -((beatmap_time as f64 - self.time as f64)*20.0).ln()*20.0 + 1.0;
        self.pos = HIT_POSITION + Vector2::new((self.time as f64 - beatmap_time as f64) * self.speed, y);
        
        if !self.done && self.pos.x - SLIDER_DOT_RADIUS <= 0.0 {
            self.done = true;
        }
    }
    pub fn draw(&self) -> [Box<dyn Renderable>; 2] {
        [
            Box::new(Circle::new(
                Color::GREEN,
                -100.0,
                self.pos,
                SLIDER_DOT_RADIUS
            )),
            // "hole punch"
            Box::new(Circle::new(
                BAR_COLOR.into(),
                0.0,
                Vector2::new(self.pos.x, HIT_POSITION.y),
                SLIDER_DOT_RADIUS
            )),
        ]
    }
}

// spinner
pub struct Spinner {
    pos: Vector2<f64>, // the note in the bar, not the spinner itself
    hit_count: u16,
    last_hit: HitType,
    complete: bool, // is this spinner done

    pub hits_required: u16, // how many hits until the spinner is "done"
    pub time: u64, // ms
    pub end_time: u64, // ms
    pub speed:f64,
    pub beatmap: Arc<Mutex<Beatmap>>,
}
impl Spinner {
    pub fn new(beatmap:Arc<Mutex<Beatmap>>, time:u64, end_time:u64, speed:f64, hits_required:u16) -> Spinner {
        Spinner {
            beatmap,
            time, 
            end_time,
            speed,
            hits_required,

            hit_count: 0,
            last_hit: HitType::Don,
            complete: false,
            pos: Vector2::new(0.0,0.0)
        }
    }
}
impl HitObject for Spinner {
    fn set_sv(&mut self, sv:f64) {self.speed = sv;}
    fn time(&self) -> u64 {self.time}
    fn end_time(&self) -> u64 {
        // if the spinner is done, end right away
        if self.complete {self.time} else {self.end_time}
    }
    fn causes_miss(&self) -> bool {!self.complete} // if the spinner wasnt completed in time, cause a miss
    fn note_type(&self) -> NoteType {NoteType::Spinner}
    fn get_points(&mut self, hit_type:HitType, time:f64) -> ScoreHit {
        // too soon or too late
        if time < self.time as f64 || time > self.end_time as f64 {return ScoreHit::None}
        // wrong note, or already done (just in case)
        if self.last_hit == hit_type || self.complete {return ScoreHit::None}

        self.last_hit = hit_type;
        self.hit_count += 1;
        
        if self.hit_count == self.hits_required {self.complete = true}

        ScoreHit::Other(100, self.complete)
    }

    fn update(&mut self, beatmap_time: i64) {
        self.pos = HIT_POSITION + Vector2::new((self.time as f64 - beatmap_time as f64) * self.speed, 0.0);
        if beatmap_time > self.end_time as i64 {
            self.complete = true;
        }
    }
    fn draw(&mut self, _args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut renderables: Vec<Box<dyn Renderable>> = Vec::new();

        // if done, dont draw anything
        if self.complete {return renderables;}

        // if its time to start hitting the spinner
        if self.pos.x <= HIT_POSITION.x {
            let color:Color = Color::YELLOW; //if self.last_hit == HitType::Don {[1.0, 0.0, 0.0, 1.0].into()} else {[0.0, 0.0, 1.0, 1.0].into()};

            // bg circle
            let bg = Circle::new(
                color,
                -10.0,
                HIT_POSITION + SPINNER_POSITION,
                SPINNER_RADIUS
            );
            renderables.push(Box::new(bg));

            // draw another circle on top which increases in radius as the counter gets closer to the reqired
            let fg = Circle::new(
                Color::WHITE,
                -11.0,
                HIT_POSITION + SPINNER_POSITION,
                SPINNER_RADIUS * (self.hit_count as f64 / self.hits_required as f64)
            );

            //TODO: draw a counter
            renderables.push(Box::new(fg));

        } else { // just draw the note on the playfield
            let h1 = HalfCircle::new(
                Color::BLUE,
                self.pos,
                self.time as f64,
                NOTE_RADIUS,
                false
            );
            renderables.push(Box::new(h1));

            let h2 = HalfCircle::new(
                Color::RED,
                self.pos,
                self.time as f64,
                NOTE_RADIUS,
                true
            );
            renderables.push(Box::new(h2));
        }
        
        renderables
    }

    fn reset(&mut self) {
        self.pos.x = 0.0;
        self.hit_count = 0;
        self.complete = false;
    }
}



#[derive(Clone, Copy, PartialEq)]
pub enum HitType {
    Don,
    Kat
}

#[derive(Clone, Copy, PartialEq)]
pub enum NoteType {
    Note,
    Slider,
    Spinner
}


// stolen from peppy, /shrug
pub fn map_difficulty_range(diff:f64, min:f64, mid:f64, max:f64) -> f64 {
    if diff > 5.0 {
        mid + (max - mid) * (diff - 5.0) / 5.0
    } else if diff < 5.0 {
        mid - (mid - min) * (5.0 - diff) / 5.0
    } else {
        mid
    }
}