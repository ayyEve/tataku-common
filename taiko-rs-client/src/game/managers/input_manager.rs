use std::{collections::HashSet, time::Instant};
use piston::input::*;

use crate::Vector2;
use crate::game::KeyModifiers;

pub struct InputManager {
    pub mouse_pos: Vector2,
    pub scroll_delta: f64,
    pub mouse_moved: bool,

    pub mouse_buttons: HashSet<MouseButton>,
    pub mouse_down: HashSet<(MouseButton, Instant)>,
    pub mouse_up: HashSet<(MouseButton, Instant)>,

    /// currently pressed keys
    keys: HashSet<Key>,
    /// keys that were pressed but waiting to be registered
    keys_down: HashSet<(Key, Instant)>,
    /// keys that were released but waiting to be registered
    keys_up: HashSet<(Key, Instant)>,
    
    text_cache: String,
    window_change_focus: Option<bool>,
    register_times: Vec<f32>
}
impl InputManager {
    pub fn new() -> InputManager {
        InputManager {
            mouse_pos: Vector2::zero(),
            scroll_delta: 0.0,
            mouse_moved: false,
            register_times: Vec::new(),

            mouse_buttons: HashSet::new(),
            mouse_down: HashSet::new(),
            mouse_up: HashSet::new(),

            keys: HashSet::new(),
            keys_down: HashSet::new(),
            keys_up:  HashSet::new(),
            
            text_cache: String::new(),

            window_change_focus: None,
        }
    }

    pub fn handle_events(&mut self, e:Event) {
        if let Some(button) = e.button_args() {
            match (button.button, button.state) {
                (Button::Keyboard(key), ButtonState::Press) => {
                    self.keys.insert(key);
                    self.keys_down.insert((key, Instant::now()));
                }
                (Button::Keyboard(key), ButtonState::Release) => {
                    self.keys.remove(&key);
                    self.keys_up.insert((key, Instant::now()));
                }
                (Button::Mouse(mb), ButtonState::Press) => {
                    self.mouse_buttons.insert(mb);
                    self.mouse_down.insert((mb, Instant::now()));
                }
                (Button::Mouse(mb), ButtonState::Release) => {
                    self.mouse_buttons.remove(&mb);
                    self.mouse_up.insert((mb, Instant::now()));
                }
                _ => {}
            }
        }

        e.mouse_cursor(|pos| {
            let new_pos = Vector2::new(pos[0], pos[1]);
            if new_pos != self.mouse_pos {self.mouse_moved = true}
            self.mouse_pos = new_pos;
        });

        e.mouse_scroll(|d| {self.scroll_delta += d[1]});
        if let Some(e) = e.text_args() {self.text_cache += &e}
        if let Some(has_focus) = e.focus_args() {self.window_change_focus = Some(has_focus)}
        // e.text(|text| println!("Typed '{}'", text));
    }

    /// is the key currently down (not up)
    pub fn key_down(&self, k:Key) -> bool {self.keys.contains(&k)}
    pub fn get_key_mods(&self) -> KeyModifiers {
        KeyModifiers {
            ctrl: self.key_down(Key::LCtrl) || self.key_down(Key::RCtrl),
            alt: self.key_down(Key::LAlt) || self.key_down(Key::RAlt),
            shift: self.key_down(Key::LShift) || self.key_down(Key::RShift),
        }
    }


    /// get all keys that were pressed, and clear the pressed list. (will be true when first checked and pressed, false after first check or when key is up)
    pub fn get_keys_down(&mut self) -> Vec<Key> {
        let mut down = Vec::new();
        for (i, time) in &self.keys_down {down.push(i.clone()); self.register_times.push(time.elapsed().as_secs_f32()*1000.0)}
        self.keys_down.clear();

        down
    }
    pub fn get_keys_up(&mut self) -> Vec<Key> {
        let mut up = Vec::new();
        for (i, time) in &self.keys_up {up.push(i.clone()); self.register_times.push(time.elapsed().as_secs_f32()*1000.0)}
        self.keys_up.clear();

        up
    }


    /// get all pressed mouse buttons, and reset the pressed array
    pub fn get_mouse_down(&mut self) -> Vec<MouseButton> {
        let mut down = Vec::new();
        for (i, time) in &self.mouse_down {down.push(i.clone()); self.register_times.push(time.elapsed().as_secs_f32()*1000.0)}
        self.mouse_down.clear();
        down
    }
    pub fn get_mouse_up(&mut self) -> Vec<MouseButton> {
        let mut up = Vec::new();
        for (i, time) in &self.mouse_up {up.push(i.clone()); self.register_times.push(time.elapsed().as_secs_f32()*1000.0)}
        self.mouse_up.clear();
        up
    }

    /// get whether the mouse was moved or not
    pub fn get_mouse_moved(&mut self) -> bool {
        let moved = self.mouse_moved;
        self.mouse_moved = false;
        moved
    }
    /// get how much the mouse wheel as scrolled (vertically) since the last check
    pub fn get_scroll_delta(&mut self) -> f64 {
        let delta = self.scroll_delta;
        self.scroll_delta = 0.0;
        delta
    }

    /// gets any text typed since the last check
    pub fn get_text(&mut self) -> String {
        let t = self.text_cache.clone();
        self.text_cache = String::new();
        t
    }

    /// get whether the window's focus has changed
    pub fn get_changed_focus(&mut self) -> Option<bool> {
        let o = self.window_change_focus.clone();
        self.window_change_focus = None;
        o
    }

    /// get the input register delay average 
    /// (min,max,avg)
    #[allow(unused)]
    pub fn get_register_delay(&mut self) -> (f32,f32,f32) {
        let mut sum = 0.0;
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        for i in self.register_times.iter() {
            sum += i;
            min = min.min(*i);
            max = max.max(*i);
        }
        sum /= self.register_times.len() as f32;
        self.register_times.clear();

        (min,max,sum)
    }
}
