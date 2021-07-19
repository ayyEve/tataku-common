use std::{rc::Rc, sync::Arc};

use parking_lot::Mutex;
use opengl_graphics::{GlGraphics, GlyphCache, Texture};
use graphics::{Context, DrawState, ImageSize, Transformed, character::CharacterCache};

use crate::{render::Color, game::Vector2};

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
    pub pos: Vector2,
    pub radius: f64,

    pub border:Option<Border>,
    spawn_time: u64,
    lifetime: u64
}
impl Circle {
    pub fn new(color:Color, depth:f64, pos:Vector2, radius:f64) -> Circle {
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
    fn get_depth(&self) -> f64 {self.depth}
    fn set_lifetime(&mut self, lifetime:u64) {self.lifetime = lifetime}
    fn get_lifetime(&self) -> u64 {self.lifetime}
    fn set_spawn_time(&mut self, time:u64) {self.spawn_time = time}
    fn get_spawn_time(&self) -> u64 {self.spawn_time}

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
    pub pos: Vector2,
    pub depth: f64,
    pub radius: f64,
    pub left_side: bool,

    spawn_time: u64,
    lifetime: u64,
}
impl HalfCircle {
    pub fn new(color: Color, pos: Vector2, depth: f64, radius: f64, left_side: bool) -> HalfCircle {
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
    fn get_depth(&self) -> f64 {self.depth}
    fn set_lifetime(&mut self, lifetime:u64) {self.lifetime = lifetime}
    fn get_lifetime(&self) -> u64 {self.lifetime}
    fn set_spawn_time(&mut self, time:u64) {self.spawn_time = time}
    fn get_spawn_time(&self) -> u64 {self.spawn_time}

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
    pub pos: Vector2,
    pub size: Vector2,
    pub border: Option<Border>,

    spawn_time: u64,
    lifetime: u64
}
impl Rectangle {
    pub fn new(color: Color, depth: f64, pos: Vector2, size: Vector2, border: Option<Border>) -> Rectangle {
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
    pub fn bounds_only(pos: Vector2, size: Vector2) -> Rectangle {
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
    pub pos: Vector2,
    pub font_size: u32,
    pub text: String,
    pub font: Arc<Mutex<GlyphCache<'static>>>,

    lifetime:u64,
    spawn_time:u64
}
impl Text {
    pub fn new(color:Color, depth:f64, pos: Vector2, font_size: u32, text: String, font: Arc<Mutex<GlyphCache<'static>>>) -> Text {
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
    pub fn measure_text(&self) -> Vector2 {
        let mut text_size = Vector2::zero();
        let mut font = self.font.lock();

        // let block_char = '█';
        // let character = font.character(self.font_size, block_char).unwrap();

        for _ch in self.text.chars() {
            let character = font.character(self.font_size, _ch).unwrap();
            text_size.x += character.advance_width();
            // text_size.y = text_size.y.max(character.offset[1]); //character.advance_height();
        }
        
        text_size
    }
    pub fn center_text(&mut self, rect:Rectangle) {
        let text_size = self.measure_text();
        self.pos = rect.pos + (rect.size - text_size)/2.0 + Vector2::new(0.0, text_size.y);
    }
}
impl Renderable for Text {
    fn get_depth(&self) -> f64 {self.depth}
    fn set_lifetime(&mut self, lifetime:u64) {self.lifetime = lifetime}
    fn get_lifetime(&self) -> u64 {self.lifetime}
    fn set_spawn_time(&mut self, time:u64) {self.spawn_time = time}
    fn get_spawn_time(&self) -> u64 {self.spawn_time}

    fn draw(&mut self, g: &mut GlGraphics, c: Context) {
        graphics::text(
            self.color.into(),
            self.font_size,
            self.text.as_str(),
            &mut *self.font.lock(),
            c.transform.trans(self.pos.x, self.pos.y),
            g
        ).unwrap();
    }
}

#[derive(Clone)]
pub struct Image {
    pub pos: Vector2,
    pub size:Vector2,
    pub depth: f64,
    pub tex: Rc<Texture>,
    
    spawn_time:u64,
    scale: Vector2
}
impl Image {
    pub fn new(pos: Vector2, depth: f64, tex:Texture, size:Vector2) -> Image {
        // let scale = Vector2::new(tex.get_width() as f64 / size.x, tex.get_height() as f64 / size.y);
        let scale = Vector2::new(size.x / tex.get_width() as f64, size.y / tex.get_height() as f64);

        Image {
            pos,
            size,
            depth,
            tex: Rc::new(tex),
            spawn_time: 0,
            scale,
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
        graphics::image(
            self.tex.as_ref(), 
            c.transform.trans(self.pos.x, self.pos.y).scale(self.scale.x, self.scale.y), 
            g);
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