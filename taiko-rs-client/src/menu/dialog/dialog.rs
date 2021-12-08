use crate::prelude::*;

/// a dialog is basically just a menu, except it does not occupy a whole gamemode, and can be drawn overtop every other menu

pub trait Dialog<G> {
    /// helpful for determining what menu this is
    fn update(&mut self, _g:&mut G) {}
    fn draw(&mut self, args:&RenderArgs, depth: &f64, list: &mut Vec<Box<dyn Renderable>>);
    fn get_bounds(&self) -> Rectangle;

    // input handlers
    fn on_mouse_move(&mut self, _pos:&Vector2, _g:&mut G) {}
    fn on_mouse_scroll(&mut self, _delta:&f64, _g:&mut G) -> bool {false}
    fn on_mouse_down(&mut self, _pos:&Vector2, _button:&MouseButton, _mods:&KeyModifiers, _g:&mut G) -> bool {false}
    fn on_mouse_up(&mut self, _pos:&Vector2, _button:&MouseButton, _mods:&KeyModifiers, _g:&mut G) -> bool {false}

    fn on_text(&mut self, _text:&String) -> bool {false}
    fn on_key_press(&mut self, _key:&Key, _mods:&KeyModifiers, _g:&mut G) -> bool {false}
    fn on_key_release(&mut self, _key:&Key, _mods:&KeyModifiers, _g:&mut G) -> bool {false}
}
