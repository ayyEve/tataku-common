use piston::{Key, MouseButton, RenderArgs};

use crate::game::{Game, KeyModifiers, Vector2};
use crate::render::{Color, Rectangle, Renderable};

pub trait Menu {
    /// helpful for determining what menu this is
    fn get_name(&self) -> &str {"none"}
    fn update(&mut self,_game:&mut Game) {}
    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>>;

    fn visibility_bg(&self, pos:Vector2, size:Vector2) -> Box<Rectangle> {
        let mut color = Color::WHITE;
        color.a = 0.6;
        Box::new(Rectangle::new(
            color,
            f64::MAX - 10.0,
            pos,
            size,
            None
        ))
    }

    // input handlers
    fn on_volume_change(&mut self) {}
    fn on_change(&mut self, _into:bool) {}// when the menu is "loaded"(into) or "unloaded"(!into)

    fn on_text(&mut self, _text:String) {}
    fn on_click(&mut self, _pos:Vector2, _button:MouseButton, _game:&mut Game) {}
    fn on_scroll(&mut self, _delta:f64, _game:&mut Game) {}
    fn on_mouse_move(&mut self, _pos:Vector2, _game:&mut Game) {}
    fn on_key_press(&mut self, _key:Key, _game:&mut Game, _mods:KeyModifiers) {}
    fn on_key_release(&mut self, _key:Key, _game:&mut Game) {}
    fn on_focus_change(&mut self, _has_focus:bool, _game:&mut Game) {}
}
