use std::{rc::Rc, sync::{Arc, Mutex}};

use cgmath::{Vector2};
use opengl_graphics::{GlGraphics, GlyphCache, Texture};
use graphics::{Context, DrawState, ImageSize, Transformed, character::CharacterCache};

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
            border: if self.border.is_some() {Some(self.border.unwrap().into())} else {None},
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
        let start_angle:f64 = if self.left_side {std::f64::consts::PI/2.0} else {std::f64::consts::PI*1.5} as f64;
        graphics::circle_arc(
            self.color.into(), 
            self.radius/2.0,
            start_angle, 
            start_angle + std::f64::consts::PI, 
            [self.pos.x, self.pos.y, self.radius,self.radius],
            c.transform.trans(-self.radius/2.0, -self.radius/2.0), 
        g);
    }
}

#[derive(Clone, Copy)]
pub struct Rectangle {
    pub color: Color,
    pub depth: f64,
    pub pos: Vector2<f64>,
    pub size: Vector2<f64>,
    pub border: Option<Border>,

    spawn_time: u64,
    lifetime: u64
}
impl Rectangle {
    pub fn new(color: Color, depth: f64, pos: Vector2<f64>, size: Vector2<f64>, border: Option<Border>) -> Rectangle {
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
    fn get_depth(&self) -> f64 {self.depth}
    fn set_lifetime(&mut self, lifetime:u64) {self.lifetime = lifetime}
    fn get_lifetime(&self) -> u64 {self.lifetime}
    fn set_spawn_time(&mut self, time:u64) {self.spawn_time = time}
    fn get_spawn_time(&self) -> u64 {self.spawn_time}

    fn draw(&mut self, g: &mut GlGraphics, c: Context) {
        let mut r = graphics::Rectangle::new(self.color.into());
        if let Some(b) = self.border {
            r.border = Some(b.into());
        }
        r.draw([
            self.pos.x, self.pos.y, 
            self.size.x, self.size.y
        ], &DrawState::default(),c.transform, g);
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

#[derive(Clone)]
pub struct Image {
    pub pos: Vector2<f64>,
    pub depth: f64,
    pub tex: Rc<Texture>,
    
    spawn_time:u64,
}

impl Image {
    pub fn new(pos: Vector2<f64>, depth: f64, tex:Texture) -> Image {
        Image {
            pos,
            depth,
            tex: Rc::new(tex),
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
        graphics::image(self.tex.as_ref(), c.transform.trans(self.pos.x, self.pos.y), g);
    }
}

/// generic border object, easier to use than importing graphics::X::Border, and creating it manually
#[derive(Clone, Copy)]
pub struct Border {
    pub color: Color,
    pub radius: f64
}
impl Border {
    pub fn new(color:Color, radius:f64) -> Self {
        Self {
            color, 
            radius
        }
    }
}
impl Into<graphics::rectangle::Border> for Border {
    fn into(self) -> graphics::rectangle::Border {
        graphics::rectangle::Border {
            color: self.color.into(),
            radius: self.radius
        }
    }
}
impl Into<graphics::ellipse::Border> for Border {
    fn into(self) -> graphics::ellipse::Border {
        graphics::ellipse::Border {
            color: self.color.into(),
            radius: self.radius
        }
    }
}