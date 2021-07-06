use std::error::Error;

use cgmath::Vector2;
use piston::{MouseButton, Key, RenderArgs};
use clipboard::{ClipboardProvider, ClipboardContext};

use crate::{game::{KeyModifiers, get_font}, menu::ScrollableItem, render::{Rectangle, Renderable, Color, Text, Border}};

#[derive(Clone)]
pub struct TextInput {
    pos: Vector2<f64>,
    size: Vector2<f64>,
    hover: bool,
    selected: bool,
    tag: String,

    placeholder: String,
    text: String,
    cursor_index: usize,
}
impl TextInput {
    pub fn new(pos:Vector2<f64>, size: Vector2<f64>, placeholder:&str, value:&str) -> TextInput {
        TextInput {
            pos, 
            size, 
            placeholder: placeholder.to_owned(),

            hover: false,
            selected: false,
            text: value.to_owned(),
            cursor_index: 0,
            tag: String::new()
        }
    }

    pub fn get_text(&self) -> String {self.text.clone()}
    pub fn set_text(&mut self, text:String) {
        self.text = text.clone();
        self.cursor_index = text.len();
    }

    fn add_letter(&mut self, c:char) {
        if self.cursor_index == self.text.len() {
            self.text.push(c);
        } else {
            self.text.insert(self.cursor_index, c);
        }

        self.cursor_index += 1;
    }
}
impl ScrollableItem for TextInput {
    fn size(&self) -> Vector2<f64> {self.size}
    fn get_pos(&self) -> Vector2<f64> {self.pos}
    fn set_pos(&mut self, pos:Vector2<f64>) {self.pos = pos}
    fn get_tag(&self) -> String {self.tag.clone()}
    fn set_tag(&mut self, tag:&str) {self.tag = tag.to_owned()}
    fn get_value(&self) -> Box<dyn std::any::Any> {Box::new(self.text.clone())}

    fn draw(&mut self, _args:RenderArgs, pos_offset:Vector2<f64>, parent_depth:f64) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        let font = get_font("main");

        let border = Rectangle::new(
            Color::WHITE,
            parent_depth + 1.0,
            self.pos+pos_offset,
            self.size, 
            Some(Border::new(if self.hover {Color::BLUE} else if self.selected {Color::RED} else {Color::BLACK}, 1.2))
        );
        list.push(Box::new(border));

        if self.text.len() > 0 {
            let text = Text::new(
                Color::BLACK,
                parent_depth + 1.0,
                self.pos+pos_offset + Vector2::new(0.0, 35.0),
                32,
                self.text.clone(),
                font.clone()
            );
            list.push(Box::new(text));
        } else {
            let text = Text::new(
                [0.2,0.2,0.2,1.0].into(),
                parent_depth + 1.0,
                self.pos+pos_offset + Vector2::new(0.0, 35.0),
                32,
                self.placeholder.clone(),
                font.clone()
            );
            list.push(Box::new(text));
        }

        let width = Text::new(
            Color::BLACK,
            parent_depth,
            self.pos+pos_offset,
            32,
            self.text.split_at(self.cursor_index).0.to_owned(),
            font.clone()
        ).measure_text().x;

        if self.selected {
            let cursor = Rectangle::new(
                Color::RED,
                parent_depth,
                self.pos+pos_offset + Vector2::new(width, 0.0),
                Vector2::new(0.7, self.size.y), 
                Some(Border::new(Color::RED, 1.2))
            );
            list.push(Box::new(cursor));
        }

        list
    }

    fn on_mouse_move(&mut self, p:Vector2<f64>) {
        self.hover = self.hover(p); //p.x > self.pos.x && p.x < self.pos.x + self.size.x && p.y > self.pos.y && p.y < self.pos.y + self.size.y;
    }
    fn on_click(&mut self, pos:Vector2<f64>, _btn:MouseButton) -> bool {

        // try to extrapolate where the mouse was clicked, and change the cursor_index to that
        if self.selected {
            if !self.hover {
                self.selected = false;
                return false;
            }

            let font = get_font("main");
            let rel_x = pos.x - self.pos.x;
            let mut str = String::new();

            for char in self.text.chars() {
                let width = Text::new(
                    Color::BLACK,
                    0.0,
                    self.pos,
                    32,
                    str.to_owned(),
                    font.clone()
                ).measure_text().x;

                if width > rel_x {
                    self.cursor_index = str.len();
                    return true;
                }

                str.push(char);
            }

            self.cursor_index = self.text.len();
            return true;
        }

        if self.hover {
            self.selected = true;
        }

        return self.hover
    }

    fn on_key_press(&mut self, key:Key, mods:KeyModifiers) -> bool {
        if !self.selected {return false}

        match key {
            Key::Left if self.cursor_index > 0 => self.cursor_index -= 1,
            Key::Right if self.cursor_index < self.text.len() => self.cursor_index += 1,
            Key::Backspace if self.cursor_index > 0 => {
                if self.cursor_index < self.text.len() {
                    self.text.remove(self.cursor_index);
                } else {
                    self.text.pop();
                }
                self.cursor_index -= 1;
            }
            
            Key::V => if mods.ctrl {
                let ctx:Result<ClipboardContext, Box<dyn Error>> = ClipboardProvider::new();
                match ctx {
                    Ok(mut ctx) => 
                        match ctx.get_contents() {
                            Ok(text) => self.set_text(text),
                            Err(e) => println!("[Clipboard] Error: {:?}", e),
                        }
                    Err(e) => println!("[Clipboard] Error: {:?}", e),
                }
            }
            _ => {}
        }

        true
    }
    fn on_text(&mut self, text:String) {
        if !self.selected {return}

        for c in text.chars() {self.add_letter(c)}
    }
}
