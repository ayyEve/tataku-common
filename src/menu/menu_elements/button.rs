use cgmath::Vector2;

use crate::{game::get_font, menu::ScrollableItem, render::{Color, Rectangle, Renderable, Text, Border}};

#[derive(Clone)]
pub struct MenuButton {
    pos: Vector2<f64>,
    size: Vector2<f64>,
    hover:bool,
    tag:String,

    text: String,
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
            if self.hover {Some(Border::new(Color::RED, 1.0))} else {None}
        );
        
        // draw text
        let mut txt = Text::new(
            Color::WHITE,
            -1.0,
            self.pos+pos_offset,
            font_size,
            self.text.clone(),
            get_font("main")
        );
        txt.center_text(r);

        list.push(Box::new(r));
        list.push(Box::new(txt));

        list
    }

    fn on_click(&mut self, _pos:Vector2<f64>, _button:piston::MouseButton) -> bool {self.hover}
    fn on_mouse_move(&mut self, pos:Vector2<f64>) {self.hover = self.hover(pos)}
}