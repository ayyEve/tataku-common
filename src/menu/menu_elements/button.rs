use cgmath::Vector2;
use graphics::rectangle::Border;

use crate::render::{Color, Rectangle, Renderable, Text};
use super::ScrollableItem;

#[derive(Clone)]
pub struct MenuButton {
    pub pos: Vector2<f64>,
    pub size: Vector2<f64>,
    pub text: String,

    pub hover:bool,

    tag:String,
}
impl MenuButton {
    pub fn new(pos: Vector2<f64>, size: Vector2<f64>, text:&str) -> MenuButton {
        MenuButton {
            pos, 
            size, 
            text: text.to_owned(),

            hover: false,
            tag: String::new()
        }
    }
    pub fn check_hover(&mut self, p:Vector2<f64>) {
        self.hover = p.x > self.pos.x && p.x < self.pos.x + self.size.x && p.y > self.pos.y && p.y < self.pos.y + self.size.y;
    }
}

impl ScrollableItem for MenuButton {
    fn size(&self) -> Vector2<f64> {self.size}
    fn get_pos(&self) -> Vector2<f64> {self.pos}
    fn set_pos(&mut self, pos:Vector2<f64>) {self.pos = pos}
    fn get_tag(&self) -> String {self.tag.clone()}
    fn set_tag(&mut self, tag:String) {self.tag = tag}

    fn draw(&mut self, _args:piston::RenderArgs, pos_offset:Vector2<f64>) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();

        let font_size:u32 = 12;
        
        // draw box
        let r = Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            0.0,
            self.pos+pos_offset,
            self.size,
            if self.hover {Some(Border {color: Color::RED.into(), radius: 1.0})} else {None}
        );
        list.push(Box::new(r));
        
        // draw text
        let mut txt = Text::new(
            Color::WHITE,
            -1.0,
            self.pos+pos_offset,
            font_size,
            self.text.clone(),
            crate::get_font("main")
        );
        txt.center_text(r);
        list.push(Box::new(txt));

        list
    }

    fn on_click(&mut self, _pos:Vector2<f64>, _button:piston::MouseButton) -> bool {self.hover}
    fn on_mouse_move(&mut self, pos:Vector2<f64>) {self.hover = self.hover(pos)}
}


/*
    fn get_depth(&self) -> f64 {
        1.0 //TODO lol
    }

    fn draw(&mut self, g: &mut opengl_graphics::GlGraphics, c: graphics::Context) {
        let font_size:u32 = 12;
        
        // draw box
        let mut r = Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            self.get_depth(),
            self.pos,
            self.size,
            None
        );
        if self.hover { //TODO: make this customizable?
            r.border = Some(Border {color: Color::RED.into(), radius: 1.0});
        }
        r.draw(g,c);

        // calc text size for centering
        let mut text_size: Vector2<f64> = Vector2::new(0.0,0.0);

        { // put this in its own scope, so it can release font before it needs to be drawn
            let font = crate::get_font("main").clone();
            let mut font = font.lock().unwrap();
            
            for ch in self.text.chars() {
                let character = font.character(font_size, ch).unwrap();
                text_size.x += character.advance_width();
                text_size.y += character.advance_height();
            }   
        }
        
        // draw text
        let mut txt = Text::new(
            Color::WHITE,
            self.get_depth() - 1.0,
            self.pos + (self.size - text_size)/2.0,
            font_size,
            self.text.clone(),
            crate::get_font("main")
        );

        txt.draw(g, c);
    }

    fn get_lifetime(&self) -> u64 {0}
    fn set_lifetime(&mut self, _lifetime:u64) {}
    fn get_spawn_time(&self) -> u64 {0}
    fn set_spawn_time(&mut self, _time:u64) {}

 */