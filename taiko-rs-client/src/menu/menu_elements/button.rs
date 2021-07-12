use crate::{WINDOW_SIZE, menu::ScrollableItem, game::{get_font, Vector2}, render::{Color, Rectangle, Renderable, Text, Border}};

const BACK_BUTTON_SIZE:Vector2 = Vector2::new(100.0, 50.0);

#[derive(Clone)]
pub struct MenuButton {
    pos: Vector2,
    size: Vector2,
    hover: bool,
    tag: String,

    text: String,
}
impl MenuButton {
    pub fn new(pos: Vector2, size: Vector2, text:&str) -> MenuButton {
        MenuButton {
            pos, 
            size, 
            text: text.to_owned(),

            hover: false,
            tag: String::new()
        }
    }

    pub fn back_button() -> MenuButton {
        MenuButton::new(Vector2::new(10.0, WINDOW_SIZE.y - (BACK_BUTTON_SIZE.y + 10.0)), BACK_BUTTON_SIZE, "Back")
    }
}

impl ScrollableItem for MenuButton {
    fn size(&self) -> Vector2 {self.size}
    fn get_pos(&self) -> Vector2 {self.pos}
    fn set_pos(&mut self, pos:Vector2) {self.pos = pos}
    fn get_tag(&self) -> String {self.tag.clone()}
    fn set_tag(&mut self, tag:&str) {self.tag = tag.to_owned()}

    fn draw(&mut self, _args:piston::RenderArgs, pos_offset:Vector2, parent_depth:f64) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        let font_size:u32 = 12;
        
        // draw box
        let r = Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            parent_depth + 1.0,
            self.pos+pos_offset,
            self.size,
            if self.hover {Some(Border::new(Color::RED, 1.0))} else {None}
        );
        
        // draw text
        let mut txt = Text::new(
            Color::WHITE,
            parent_depth,
            self.pos+pos_offset,
            font_size,
            self.text.clone(),
            get_font("main")
        );
        txt.center_text(r);

        list.push(Box::new(r));
        list.push(Box::new(txt));

        list
    }

    fn on_click(&mut self, _pos:Vector2, _button:piston::MouseButton) -> bool {self.hover}
    fn on_mouse_move(&mut self, pos:Vector2) {self.hover = self.hover(pos)}
}