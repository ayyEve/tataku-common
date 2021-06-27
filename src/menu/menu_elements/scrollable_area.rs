use std::any::Any;
use std::sync::{Arc, Mutex};

use cgmath::Vector2;
use piston::{Key, MouseButton, RenderArgs};

use crate::game::{Game, KeyModifiers};
use crate::render::*;


// how many pixels should be between items when in list mode?
const ITEM_MARGIN:f64 = 5.0;

/// how much should a scroll unit be worth?
const SCROLL_FACTOR: f64 = 8.0;

pub struct ScrollableArea {
    items: Vec<Box<dyn ScrollableItem>>,
    scroll_pos: f64,
    pos: Vector2<f64>,
    size: Vector2<f64>,
    /// if list mode, item positions will be modified based on how many items there are (ie, a list)
    list_mode: bool,

    // cache of where the mouse is, needed to check for on_scroll if mouse is over this
    mouse_pos: Vector2<f64>,
    elements_height: f64,
}
impl ScrollableArea {
    pub fn new(pos: Vector2<f64>, size: Vector2<f64>, list_mode: bool) -> ScrollableArea {
        ScrollableArea {
            items: Vec::new(),
            scroll_pos: 0.0,
            pos,
            size,
            list_mode,
            elements_height: 0.0,

            mouse_pos: Vector2::new(-999.0,-999.0) // just in case lol
        }
    }

    pub fn update(&mut self) {} //TODO: maybe set this up lol
    pub fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut items: Vec<Box<dyn Renderable>> = Vec::new();

        let last_y = if self.items.last().is_some() {self.items.last().unwrap().size().y} else {0.0};
        let min = -(self.elements_height + last_y) + self.size.y;
        let max = 0.0;
        if !(min>max) {
            self.scroll_pos = self.scroll_pos.clamp(min, max);
        } else {
            self.scroll_pos = 0.0;
        }

        for item in self.items.as_mut_slice() {
            items.extend(item.draw(args, Vector2::new(0.0, self.scroll_pos)));
        }

        // helpful for debugging positions
        // items.push(Box::new(Rectangle::new(Color::TRANSPARENT_WHITE, -10.0, self.pos, self.size, Some(Border {color:Color::BLACK.into(), radius: 2.0}))));
        // let pos = self.mouse_pos - Vector2::new(0.0, self.scroll_pos);
        // items.push(Box::new(Circle::new(Color::BLUE, -10.0, pos, 5.0)));

        items
    }

    // input handlers
    /// returns the tag of the item which was clicked
    pub fn on_click(&mut self, pos:Vector2<f64>, button:MouseButton, _game:Arc<Mutex<&mut Game>>) -> Option<String> {

        // modify pos here
        let pos = pos - Vector2::new(0.0, self.scroll_pos);
        let mut i = None;

        for item in self.items.as_mut_slice() {
            if item.on_click(pos, button) {
                // return;
                i = Some(item.get_tag());
            }
        }

        i
    }
    pub fn on_mouse_move(&mut self, pos:Vector2<f64>, _game:Arc<Mutex<&mut Game>>) {
        self.mouse_pos = pos;

        let pos = pos-Vector2::new(0.0, self.scroll_pos);
        for item in self.items.as_mut_slice() {
            item.on_mouse_move(pos);
        }
    }

    pub fn on_scroll(&mut self, delta:f64) {
        self.scroll_pos += delta * SCROLL_FACTOR;
    }
    pub fn on_key_press(&mut self, key:Key, mods:KeyModifiers) {
        for item in self.items.as_mut_slice() {
            item.on_key_press(key, mods);
        }
    }
    
    pub fn on_text(&mut self, text:String) {
        for item in self.items.as_mut_slice() {
            item.on_text(text.clone());
        }
    }
    
    /// returns index
    pub fn add_item(&mut self, mut item:Box<dyn ScrollableItem>) -> usize {
        if self.list_mode {
            self.elements_height += item.size().y + ITEM_MARGIN;

            let ipos = item.get_pos();
            item.set_pos(self.pos + Vector2::new(ipos.x, self.elements_height));
        }

        self.items.push(item);
        self.items.len() - 1
    }
    pub fn remove_item(&mut self, i:usize) {
        self.items.remove(i);
    }
    pub fn clear(&mut self) {
        self.items.clear();
        self.elements_height = 0.0;
    }
    pub fn get_tagged(&self, tag:String) -> Vec<&Box<dyn ScrollableItem>> {
        let mut list = Vec::new();
        for i in self.items.as_slice() {
            if i.get_tag() == tag {
                list.push(i.to_owned());
            }
        }

        list
    }

    /// completely refresh the positions for all items in the list (only effective when in list mode)
    pub fn refresh_layout(&mut self) {
        if !self.list_mode {return}

        self.elements_height = 0.0;

        for item in self.items.as_mut_slice() {
            let ipos = item.get_pos();
            item.set_pos(self.pos + Vector2::new(ipos.x, self.elements_height));
            self.elements_height += item.size().y + ITEM_MARGIN;
        }
    }
}


pub trait ScrollableItem {
    fn update(&mut self) {}
    fn size(&self) -> Vector2<f64>;

    fn get_tag(&self) -> String;
    fn set_tag(&mut self, tag:String);

    fn get_pos(&self) -> Vector2<f64>;
    fn set_pos(&mut self, pos:Vector2<f64>);
    fn hover(&self, p:Vector2<f64>) -> bool {
        let pos = self.get_pos();
        let size = self.size();
        p.x > pos.x && p.x < pos.x + size.x && p.y > pos.y && p.y < pos.y + size.y
    }

    fn draw(&mut self, args:RenderArgs, pos_offset:Vector2<f64>) -> Vec<Box<dyn Renderable>>;

    // input handlers

    /// when the mouse is clicked
    fn on_click(&mut self, pos:Vector2<f64>, button:MouseButton) -> bool; // this should be handled

    /// when the mouse is moved
    fn on_mouse_move(&mut self, pos:Vector2<f64>);

    /// when text is input
    fn on_text(&mut self, _text:String) {}

    /// when a key is pressed
    fn on_key_press(&mut self, _key:Key, _mods:KeyModifiers) -> bool {false}
    
    /// when a key is released TODO!()
    fn on_key_release(&mut self, _key:Key) {}

    /// get a value
    fn get_value(&self) -> Box<dyn Any> {Box::new(0)}
}
