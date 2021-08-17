use std::{collections::HashSet, time::Instant};
use piston::input::*;
use crate::Vector2;
pub use ayyeve_piston_ui::menu::KeyModifiers;

pub struct InputManager {
    pub mouse_pos: Vector2,
    pub scroll_delta: f64,
    pub mouse_moved: bool,
    pub mouse_buttons: Vec<MouseButton>,

    key_states: HashSet<Key>,
    key_states_once: HashSet<(Key, Instant)>,
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
            mouse_buttons: Vec::new(),
            register_times: Vec::new(),

            key_states: HashSet::new(),
            key_states_once: HashSet::new(),
            text_cache: String::new(),

            window_change_focus: None,
        }
    }

    pub fn handle_events(&mut self, e:Event) {
        if let Some(button) = e.button_args() {
            match (button.button, button.state) {
                (Button::Keyboard(key), ButtonState::Press) => {
                    self.key_states.insert(key);
                    self.key_states_once.insert((key, Instant::now()));
                },

                (Button::Keyboard(key), ButtonState::Release) => {self.key_states.remove(&key);},
                (Button::Mouse(mb), ButtonState::Press) => self.mouse_buttons.push(mb),
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
    pub fn key_down(&self, k:Key) -> bool {self.key_states.contains(&k)}

    pub fn get_key_mods(&self) -> KeyModifiers {
        KeyModifiers {
            ctrl: self.key_down(Key::LCtrl) || self.key_down(Key::RCtrl),
            alt: self.key_down(Key::LAlt) || self.key_down(Key::RAlt),
            shift: self.key_down(Key::LShift) || self.key_down(Key::RShift),
        }
    }


    /// get all keys that were pressed, and clear the pressed list. (will be true when first checked and pressed, false after first check or when key is up)
    pub fn all_down_once(&mut self) -> Vec<Key> {
        let mut down = Vec::new();
        for (i, time) in &self.key_states_once {down.push(i.clone()); self.register_times.push(time.elapsed().as_secs_f32()*1000.0)}
        self.key_states_once.clear();

        down
    }

    /// get all pressed mouse buttons, and reset the pressed array
    pub fn get_mouse_buttons(&mut self) -> Vec<MouseButton> {
        let buttons = self.mouse_buttons.clone();
        self.mouse_buttons.clear();
        buttons
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
