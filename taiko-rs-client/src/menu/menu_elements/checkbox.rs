use crate::{menu::ScrollableItem,game::{get_font, Vector2},  render::{Color, Rectangle, Renderable, Text, Border}};

const INNER_BOX_PADDING:f64 = 8.0;

#[derive(Clone)]
pub struct Checkbox {
    pos: Vector2,
    size: Vector2,
    hover:bool,
    tag: String,

    text: String,
    checked: bool
}
impl Checkbox {
    pub fn new(pos: Vector2, size: Vector2, text:&str, value:bool) -> Checkbox {
        Checkbox {
            pos, 
            size, 
            text: text.to_owned(),

            hover: false,
            tag: String::new(),
            checked: value
        }
    }
}

impl ScrollableItem for Checkbox {
    fn size(&self) -> Vector2 {self.size}
    fn get_pos(&self) -> Vector2 {self.pos}
    fn set_pos(&mut self, pos:Vector2) {self.pos = pos}
    fn get_tag(&self) -> String {self.tag.clone()}
    fn set_tag(&mut self, tag:&str) {self.tag = tag.to_owned()}
    fn get_value(&self) -> Box<dyn std::any::Any> {Box::new(self.checked)}

    fn draw(&mut self, _args:piston::RenderArgs, pos_offset:Vector2, parent_depth:f64) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        let font_size:u32 = 12;
        
        // draw bounding box
        list.push(Box::new(Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            parent_depth,
            self.pos+pos_offset,
            self.size,
            if self.hover {Some(Border::new(Color::RED, 1.0))} else {None}
        )));

        // draw checkbox bounding box
        list.push(Box::new(Rectangle::new(
            Color::TRANSPARENT_BLACK,
            parent_depth + 1.0,
            self.pos+pos_offset,
            Vector2::new(self.size.y, self.size.y),
            if self.hover {Some(Border::new(Color::BLACK, 1.0))} else {None}
        )));
        if self.checked {
            list.push(Box::new(Rectangle::new(
                Color::YELLOW,
                parent_depth,
                self.pos+pos_offset + Vector2::new(INNER_BOX_PADDING, INNER_BOX_PADDING),
                Vector2::new(self.size.y-INNER_BOX_PADDING*2.0, self.size.y-INNER_BOX_PADDING * 2.0),
                None
            )));
        }
        
        // draw text
        let mut txt = Text::new(
            Color::WHITE,
            parent_depth - 1.0,
            self.pos+pos_offset,
            font_size,
            self.text.clone(),
            get_font("main")
        );
        txt.center_text(Rectangle::bounds_only(self.pos+pos_offset + Vector2::new(self.size.y, 0.0), Vector2::new(self.size.x - self.size.y, self.size.y)));

        list.push(Box::new(txt));
        list
    }

    fn on_click(&mut self, _pos:Vector2, _button:piston::MouseButton) -> bool {
        if self.hover {self.checked = !self.checked}
        self.hover
    }
    fn on_mouse_move(&mut self, pos:Vector2) {self.hover = self.hover(pos)}
}