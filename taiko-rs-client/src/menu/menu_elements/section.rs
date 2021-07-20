

use crate::{game::Vector2, render::{Color, Rectangle, Text}};
use super::ScrollableItem;


/// basically a spacer with some text
pub struct MenuSection {
    pos:Vector2,
    text: String,
    height: f64,

    // hover: bool
}
impl MenuSection {
    pub fn new(pos:Vector2, height:f64, text:&str) -> Self {
        Self {
            pos, 
            height,
            text: text.to_owned(),
            // hover: false
        }
    }
}

impl ScrollableItem for MenuSection {
    fn set_tag(&mut self, _tag:&str) {}
    fn get_tag(&self) -> String {String::new()}
    fn size(&self) -> crate::game::Vector2 {Vector2::new(300.0, self.height)}
    fn on_click(&mut self, _pos:crate::game::Vector2, _button:piston::MouseButton) -> bool {false} //{self.hover}
    fn get_pos(&self) -> crate::game::Vector2 {self.pos}
    fn set_pos(&mut self, pos:crate::game::Vector2) {self.pos = pos}


    fn get_hover(&self) -> bool {false}
    fn set_hover(&mut self, _:bool) {}
    fn get_selected(&self) -> bool {false}
    fn set_selected(&mut self, _:bool) {}
    fn get_selectable(&self) -> bool {false}


    fn draw(&mut self, _args:piston::RenderArgs, pos_offset:crate::game::Vector2, parent_depth:f64) -> Vec<Box<dyn crate::render::Renderable>> {
        const TEXT_OFFSET:f64 = 20.0;

        let y = self.height - TEXT_OFFSET;

        let t = Text::new(
            Color::BLACK,
            parent_depth,
            self.pos + pos_offset + Vector2::new(0.0, y),
            32,
            self.text.clone(),
            crate::game::get_font("main")
        );
        let r = Rectangle::new(
            Color::BLACK,
            parent_depth,
            self.pos + pos_offset + Vector2::new(0.0, y + 10.0),
            Vector2::new(self.size().x, 4.0),
            None
        );

        let mut list:Vec<Box<dyn crate::render::Renderable>> = Vec::new();
        list.push(Box::new(t));
        list.push(Box::new(r));
        list
    }

}