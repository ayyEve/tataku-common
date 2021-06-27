use std::sync::{Arc, Mutex};

use cgmath::Vector2;
use piston::{Key, MouseButton, RenderArgs};

use crate::render::Renderable;
use crate::game::{Game, KeyModifiers};

pub trait Menu {
    fn update(&mut self, _game:Arc<Mutex<&mut Game>>) {}
    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>>;

    // input handlers
    fn on_volume_change(&mut self) {}
    fn on_change(&mut self){}// when the menu is "loaded"

    fn on_text(&mut self, _text:String){}
    fn on_click(&mut self, _pos:Vector2<f64>, _button:MouseButton, _game:Arc<Mutex<&mut Game>>) {}
    fn on_scroll(&mut self, _delta:f64, _game:Arc<Mutex<&mut Game>>) {}
    fn on_mouse_move(&mut self, _pos:Vector2<f64>, _game:Arc<Mutex<&mut Game>>) {}
    fn on_key_press(&mut self, _key:Key, _game:Arc<Mutex<&mut Game>>, _mods:KeyModifiers) {}
    fn on_key_release(&mut self, _key:Key, _game:Arc<Mutex<&mut Game>>) {}
}
