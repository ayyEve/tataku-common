use std::collections::HashSet;

use cgmath::Vector2;
use piston::input::*;

pub struct InputManager {
    pub mouse_pos: Vector2<f64>,
    pub scroll_delta: f64,
    pub mouse_moved: bool,
    pub mouse_buttons: Vec<MouseButton>,

    key_states: HashSet<Key>,
    key_states_once: HashSet<Key>,
    text_cache: String
}
impl InputManager {
    pub fn new() -> InputManager {
        InputManager {
            mouse_pos: Vector2::new(0.0, 0.0),
            scroll_delta: 0.0,
            mouse_moved: false,
            mouse_buttons: Vec::new(),

            key_states: HashSet::new(),
            key_states_once: HashSet::new(),
            text_cache: String::new()
        }
    }

    pub fn handle_events(&mut self, e:Event) {
        if let Some(button) = e.button_args() {
            match button.button {
                Button::Keyboard(key) => {
                    match button.state {
                        ButtonState::Press => {
                            self.key_states.insert(key);
                            self.key_states_once.insert(key);
                        },
                        ButtonState::Release => {
                            self.key_states.remove(&key);
                        }
                    }
                },
                Button::Mouse(mb) => {
                    match button.state {
                        ButtonState::Press => {
                            self.mouse_buttons.push(mb);
                        },
                        ButtonState::Release => {}, // for this game, we dont care about holds, so consume the mouse click when its checked for
                    }
                },
                Button::Controller(_) => {},
                Button::Hat(_) => {},
            }
        }

        if let Some(e) = e.text_args() {
            self.text_cache += &e;
        }

        e.mouse_cursor(|pos| {
            let new_pos:Vector2<f64> = Vector2::new(pos[0], pos[1]);
            
            if new_pos != self.mouse_pos {
                self.mouse_moved = true;
            }

            self.mouse_pos = new_pos;
        });
        e.mouse_scroll(|d| {
            self.scroll_delta += d[1];
        });
        // e.text(|text| println!("Typed '{}'", text));
    }

    /// is the key currently down (not up)
    pub fn key_down(&self, k:Key) -> bool{
        self.key_states.contains(&k)
    }

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

        for i in &self.key_states_once {
            down.push(i.clone());
        }
        self.key_states_once.clear();

        down
    }


    /// gets any text typed since the last check
    pub fn get_text(&mut self) -> String {
        let t = self.text_cache.clone();
        self.text_cache = String::new();
        t
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct KeyModifiers {
    pub alt: bool,
    pub ctrl: bool,
    pub shift: bool
}
