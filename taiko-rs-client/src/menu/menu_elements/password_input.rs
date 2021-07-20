use std::error::Error;

use piston::{Key, MouseButton, RenderArgs};
use clipboard::{ClipboardProvider, ClipboardContext};

use crate::{game::{KeyModifiers, get_font, Vector2}, menu::ScrollableItem, render::{Color, Rectangle, Renderable, Text, Border}};

#[derive(Clone)]
pub struct PasswordInput {
    pos: Vector2,
    size: Vector2,
    hover: bool,
    selected: bool,
    tag: String,

    placeholder: String,
    text: String,
    cursor_index: usize,
    show_pass: bool,
}
impl PasswordInput {
    pub fn new(pos: Vector2, size: Vector2, placeholder: &str, value:&str) -> PasswordInput {
        PasswordInput {
            pos, 
            size, 
            placeholder: placeholder.to_owned(),

            hover: false,
            selected: false,
            text: value.to_owned(),
            cursor_index: 0,
            show_pass: false,
            tag:String::new()
        }
    }

    // pub fn get_text(&self) -> String {self.text.clone()}
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
impl ScrollableItem for PasswordInput {
    fn size(&self) -> Vector2 {self.size}
    fn get_pos(&self) -> Vector2 {self.pos}
    fn set_pos(&mut self, pos:Vector2) {self.pos = pos;}
    fn get_tag(&self) -> String {self.tag.clone()}
    fn set_tag(&mut self, tag:&str) {self.tag = tag.to_owned()}
    fn get_value(&self) -> Box<dyn std::any::Any> {Box::new(self.text.clone())}

    fn get_hover(&self) -> bool {self.hover}
    fn set_hover(&mut self, hover:bool) {self.hover = hover; self.show_pass = hover}
    fn get_selected(&self) -> bool {self.selected}
    fn set_selected(&mut self, selected:bool) {self.selected = selected}

    fn draw(&mut self, _args:RenderArgs, pos_offset:Vector2, parent_depth:f64) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        let font = get_font("main");

        let border = Rectangle::new(
            Color::WHITE,
            parent_depth + 1.0,
            self.pos + pos_offset,
            self.size, 
            Some(Border::new(if self.hover {Color::BLUE} else if self.selected {Color::RED} else {Color::BLACK}, 1.2))
        );
        list.push(Box::new(border));

        let text = if self.show_pass {self.text.clone()} else {"*".repeat(self.text.len())};

        if self.text.len() > 0 {
            let text = Text::new(
                Color::BLACK,
                parent_depth + 1.0,
                self.pos + pos_offset + Vector2::new(0.0, 35.0),
                32,
                text.clone(),
                font.clone()
            );
            list.push(Box::new(text));
        } else {
            let text = Text::new(
                [0.2,0.2,0.2,1.0].into(),
                parent_depth + 1.0,
                self.pos + pos_offset + Vector2::new(0.0, 35.0),
                32,
                self.placeholder.clone(),
                font.clone()
            );
            list.push(Box::new(text));
        }

        let width = Text::new(
            Color::BLACK,
            parent_depth,
            self.pos,
            32,
            text.clone().split_at(self.cursor_index).0.to_owned(),
            font.clone()
        ).measure_text().x;

        if self.selected {
            let cursor = Rectangle::new(
                Color::RED,
                parent_depth,
                self.pos + pos_offset + Vector2::new(width, 0.0),
                Vector2::new(0.7, self.size.y), 
                Some(Border::new(Color::RED, 1.2))
            );
            list.push(Box::new(cursor));
        }

        list
    }

    fn on_click(&mut self, pos:Vector2, _button:MouseButton) -> bool {
        self.show_pass = false;

        // try to extrapolate where the mouse was clicked, and change the cursor_index to that
        if self.selected {
            if !self.hover {
                self.selected = false;
                return false;
            }

            let font = get_font("main");
            let rel_x = pos.x - self.pos.x;
            let mut str = String::new();

            let text = if self.show_pass {self.text.clone()} else {"*".repeat(self.text.len())};
            for char in text.chars() {
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
        self.show_pass = false;

        if !self.selected {return false;}
        if mods.alt {
            self.show_pass = true;
            return true;
        }

        match key {
            Key::Left => {
                if self.cursor_index > 0 {
                    self.cursor_index -= 1;
                }
            }
            Key::Right => {
                if self.cursor_index < self.text.len() {
                    self.cursor_index += 1;
                }
            }
            Key::Backspace => {
                if self.cursor_index > 0 {
                    if self.cursor_index < self.text.len() {

                        self.text.remove(self.cursor_index);
                    } else {
                        self.text.pop();
                    }
                    self.cursor_index -= 1;
                }
            }
            
            Key::V => if mods.ctrl {
                let ctx:Result<ClipboardContext, Box<dyn Error>> = ClipboardProvider::new();
                match ctx {
                    Ok(mut ctx) => 
                        match ctx.get_contents() {
                            Ok(text) => self.set_text(text),//for c in text.chars() {self.add_letter(c);},
                            Err(e) => println!("[Clipboard] Error: {:?}", e),
                        }
                    Err(e) => println!("[Clipboard] Error: {:?}", e),
                }
            }
            _ => {}
        }

        true
    }

    // fn on_mouse_move(&mut self, p:Vector2) {
    //     self.show_pass = false;
    //     self.hover = self.hover(p);
    // }

    fn on_text(&mut self, text:String) {
        if !self.selected {return}

        for c in text.chars() {
            self.add_letter(c);
        }
    }
}
