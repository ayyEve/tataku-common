use cgmath::Vector2;
use piston::{Key, MouseButton, RenderArgs};

use crate::{menu::ScrollableItem, game::{KeyModifiers,get_font}, render::{Color, Rectangle, Renderable, Text, Border}};

#[derive(Clone)]
pub struct KeyButton {
    pos: Vector2<f64>,
    size: Vector2<f64>,
    selected: bool,
    hover: bool,
    tag:String,

    key: Key,
    prefix: String,
}
impl KeyButton {
    pub fn new(pos: Vector2<f64>, size: Vector2<f64>, key:Key, prefix: &str) -> KeyButton {
        KeyButton {
            key,
            pos, 
            size, 
            prefix: prefix.to_owned(),

            hover: false,
            selected: false,
            tag: String::new()
        }
    }

    fn text(&self) -> String {
        if self.selected {
            "Press a key".to_owned()
        } else {
            format!("{:?}", self.key)
        }
    }
}
impl ScrollableItem for KeyButton {
    fn size(&self) -> Vector2<f64> {self.size}
    fn get_pos(&self) -> Vector2<f64> {self.pos}
    fn set_pos(&mut self, pos:Vector2<f64>) {self.pos = pos;}
    fn get_tag(&self) -> String {self.tag.clone()}
    fn set_tag(&mut self, tag:String) {self.tag = tag}
    fn get_value(&self) -> Box<dyn std::any::Any> {Box::new(self.key.clone())}

    fn draw(&mut self, _args:RenderArgs, pos_offset:Vector2<f64>) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        let font = get_font("main");

        let border = Rectangle::new(
            Color::WHITE,
            1.0,
            self.pos+pos_offset,
            self.size, 
            Some(Border::new(if self.hover {Color::BLUE} else if self.selected {Color::RED} else {Color::BLACK}, 1.2))
        );
        list.push(Box::new(border));

        let text = Text::new(
            Color::BLACK,
            1.0,
            self.pos+pos_offset + Vector2::new(0.0, 35.0),
            32,
            format!("{}: {}", self.prefix, self.text()),
            font.clone()
        );
        list.push(Box::new(text));

        list
    }

    fn on_mouse_move(&mut self, p:Vector2<f64>) {
        self.hover = p.x > self.pos.x && p.x < self.pos.x + self.size.x && p.y > self.pos.y && p.y < self.pos.y + self.size.y;
    }

    fn on_click(&mut self, _pos:Vector2<f64>, _btn:MouseButton) -> bool {

        // try to extrapolate where the mouse was clicked, and change the cursor_index to that
        if self.selected {
            if !self.hover {
                self.selected = false;

                return false;
            }
            return true;
        }

        if self.hover {
            self.selected = true;
        }

        return self.hover
    }

    fn on_key_press(&mut self, key:Key, _mods:KeyModifiers) -> bool {
        if !self.selected {
            return false;
        }

        // TODO: check exclusion list
        if key == Key::Escape {
            self.selected = false;
            return true;
        }

        self.key = key;
        self.selected = false;

        true
    }
}
