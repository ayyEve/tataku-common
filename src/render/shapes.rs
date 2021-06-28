use std::sync::{Arc, Mutex};

use opengl_graphics::{GlGraphics, GlyphCache, Texture};
use graphics::{Context, DrawState, ImageSize, Transformed, character::CharacterCache, ellipse::Border, types::Polygon};
use cgmath::{Vector2};

use crate::render::Color;

pub trait Renderable {
    fn get_depth(&self) -> f64;
    fn get_lifetime(&self) -> u64;
    fn set_lifetime(&mut self, lifetime:u64);
    fn get_spawn_time(&self) -> u64;
    fn set_spawn_time(&mut self, time:u64);
    fn draw(&mut self, g: &mut GlGraphics, c:Context);
}

#[derive(Clone, Copy)]
pub struct Circle {
    pub color: Color,
    pub depth: f64,
    pub pos: Vector2<f64>,
    pub radius: f64,

    pub border:Option<Border>,
    spawn_time: u64,
    lifetime: u64
}
impl Circle {
    pub fn new(color:Color, depth:f64, pos:Vector2<f64>, radius:f64) -> Circle {
        Circle {
            color,
            depth,
            pos,
            radius,
            border: None,
            spawn_time: 0,
            lifetime: 0
        }
    }
}
impl Renderable for Circle {
    fn get_depth(&self) -> f64 {
        self.depth
    }
    fn set_lifetime(&mut self, lifetime:u64) {
        self.lifetime = lifetime;
    }
    fn get_lifetime(&self) -> u64 {
        self.lifetime
    }
    fn set_spawn_time(&mut self, time:u64) {
        self.spawn_time = time;
    }
    fn get_spawn_time(&self) -> u64 {
       self.spawn_time
    }

    fn draw(&mut self, g: &mut GlGraphics, c: Context) {

        graphics::ellipse::Ellipse {
            color: self.color.into(),
            border: self.border,
            resolution: 128
        }.draw(
            graphics::ellipse::circle(self.pos.x, self.pos.y, self.radius),
            &DrawState::default(),
            c.transform,
            g
        );
    }
}

#[derive(Clone, Copy)]
pub struct HalfCircle {
    pub color: Color,
    pub pos: Vector2<f64>,
    pub depth: f64,
    pub radius: f64,
    pub left_side: bool,

    spawn_time: u64,
    lifetime: u64,
}
impl HalfCircle {
    pub fn new(color: Color, pos: Vector2<f64>, depth: f64, radius: f64, left_side: bool) -> HalfCircle {
        HalfCircle {
            color,
            pos,
            depth,
            radius,
            left_side,

            spawn_time:0,
            lifetime:0
        }
    }
}
impl Renderable for HalfCircle {
    fn get_depth(&self) -> f64 {
        self.depth
    }
    fn set_lifetime(&mut self, lifetime:u64) {
        self.lifetime = lifetime;
    }
    fn get_lifetime(&self) -> u64 {
        self.lifetime
    }
    fn set_spawn_time(&mut self, time:u64) {
        self.spawn_time = time;
    }
    fn get_spawn_time(&self) -> u64 {
       self.spawn_time
    }

    fn draw(&mut self, g: &mut GlGraphics, c: Context) {

        let start_angle:f64 = (if self.left_side {90f64} else {270f64}).to_radians();
        let points = polygon(self.pos, 100, self.radius, start_angle, true);
        let polygon:Polygon = points.as_slice(); //points.split_at(points.len()/2).0;

        graphics::polygon(
            self.color.into(),
            polygon,
            c.transform,
            g
        );

        // graphics::ellipse(
        //     self.color.into(),
        //     graphics::ellipse::circle(self.pos.x, self.pos.y, self.radius),
        //     transform,
        //     g
        // );

    }
}

#[derive(Clone, Copy)]
pub struct Rectangle {
    pub color: Color,
    pub depth: f64,
    pub pos: Vector2<f64>,
    pub size: Vector2<f64>,
    pub border: Option<graphics::rectangle::Border>,

    spawn_time: u64,
    lifetime: u64
}
impl Rectangle {
    pub fn new(color: Color, depth: f64, pos: Vector2<f64>, size: Vector2<f64>, border: Option<graphics::rectangle::Border>) -> Rectangle {
        Rectangle {
            color,
            depth,
            pos,
            size,
            border,

            spawn_time: 0,
            lifetime: 0
        }
    }
    
    /// helpful shortcut when you only want to measure text
    pub fn bounds_only(pos: Vector2<f64>, size: Vector2<f64>) -> Rectangle {
        Rectangle::new(Color::BLACK, 0.0, pos, size, None)
    }
}
impl Renderable for Rectangle {
    fn get_depth(&self) -> f64 {
        self.depth
    }
    fn set_lifetime(&mut self, lifetime:u64) {
        self.lifetime = lifetime;
    }
    fn get_lifetime(&self) -> u64 {
        self.lifetime
    }
    fn set_spawn_time(&mut self, time:u64) {
        self.spawn_time = time;
    }
    fn get_spawn_time(&self) -> u64 {
       self.spawn_time
    }

    fn draw(&mut self, g: &mut GlGraphics, c: Context) {

        let mut r=graphics::Rectangle::new(self.color.into());
        r.border = self.border;
        r.draw([
            self.pos.x, self.pos.y, 
            self.size.x,self.size.y
        ], &DrawState::default(),c.transform, g);

        // graphics::rectangle(
        //     self.color.into(),
            
        //     c.transform,
        //     g
        // );

        // if let Some(color) = self.border_color {
        //     //TODO
        //     graphics::Rectangle::border(self, value)
        // }

    }
}

#[derive(Clone)]
pub struct Text {
    pub color: Color,
    pub depth: f64,
    pub pos: Vector2<f64>,
    pub font_size: u32,
    pub text: String,
    pub font: Arc<Mutex<GlyphCache<'static>>>,

    lifetime:u64,
    spawn_time:u64
}
impl Text {
    pub fn new(color:Color, depth:f64, pos: Vector2<f64>, font_size: u32, text: String, font: Arc<Mutex<GlyphCache<'static>>>) -> Text {
        Text {
            color,
            depth,
            pos,
            font_size,
            text,
            font,
            lifetime: 0,
            spawn_time: 0
        }
    }
    pub fn measure_text(&self) -> Vector2<f64> {
        let mut text_size: Vector2<f64> = Vector2::new(0.0,0.0);
        let mut font = self.font.lock().unwrap();
        
        for ch in self.text.chars() {
            let character = font.character(self.font_size, ch).unwrap();
            text_size.x += character.advance_width();
            text_size.y += character.advance_height();
        }
        
        text_size
    }
    pub fn center_text(&mut self, rect:Rectangle) {
        let text_size = self.measure_text();
        self.pos = rect.pos + (rect.size/2.0 - text_size/2.0);
    }
}
impl Renderable for Text {
    fn get_depth(&self) -> f64 {
        self.depth
    }
    fn set_lifetime(&mut self, lifetime:u64) {
        self.lifetime = lifetime;
    }
    fn get_lifetime(&self) -> u64 {
        self.lifetime
    }
    fn set_spawn_time(&mut self, time:u64) {
        self.spawn_time = time;
    }
    fn get_spawn_time(&self) -> u64 {
       self.spawn_time
    }

    fn draw(&mut self, g: &mut GlGraphics, c: Context) {
        graphics::text(
            self.color.into(),
            self.font_size,
            self.text.as_str(),
            &mut *self.font.lock().unwrap(),
            c.transform.trans(self.pos.x, self.pos.y),
            g
        ).unwrap();
    }
}


pub struct Image {
    pub pos: Vector2<f64>,
    pub depth: f64,
    pub tex: Texture,
    
    spawn_time:u64,
}
impl Image {
    pub fn new(pos: Vector2<f64>, depth: f64, tex:Texture) -> Image {
        Image {
            pos,
            depth,
            tex,
            spawn_time: 0,
        }
    }
}
impl Renderable for Image {
    fn get_lifetime(&self) -> u64 {0}
    fn set_lifetime(&mut self, _lifetime:u64) {}
    fn get_spawn_time(&self) -> u64 {self.spawn_time}
    fn set_spawn_time(&mut self, time:u64) {self.spawn_time = time}
    fn get_depth(&self) -> f64 {self.depth}
    fn draw(&mut self, g: &mut GlGraphics, c: Context) {
        graphics::image(&self.tex, c.transform.trans(self.pos.x, self.pos.y), g);
    }
}
impl Clone for Image {
    fn clone(&self) -> Self {
        Image::new(self.pos, self.depth, Texture::new(self.tex.get_id(), self.tex.get_width(), self.tex.get_height()))
    }
}

/// create a polygon with coords at `pos`, `point_count` points, `radius` pixels, `angle` starting angle. 
/// `half` specifies whether to only create half the polygod
fn polygon(pos: Vector2<f64>, mut point_count:usize, radius:f64, mut angle:f64, half:bool) -> Vec<[f64;2]> {

    let mut points:Vec<[f64;2]> = Vec::new();
    let angle_diff = (360.0 / point_count as f64).to_radians();

    // for(var theta=0;  theta < 2*Math.PI;  theta+=step){ 
    //     var x = h + r*Math.cos(theta);
    //     var y = k - r*Math.sin(theta);    //note 2.
    //     ctx.lineTo(x,y);
    // }
    if half {
        point_count /= 2;
    }

    while point_count > 0 {
        point_count -= 1;
        let x = pos.x + angle.cos() * radius;
        let y = pos.y + angle.sin() * radius;
        points.push([x,y]);
        angle += angle_diff;
    }

    points
}
