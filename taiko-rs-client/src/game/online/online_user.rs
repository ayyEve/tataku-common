
use crate::render::*;
use crate::{game::get_font, Vector2};
use crate::menu::ScrollableItem;
use ayyeve_piston_ui::menu::KeyModifiers;
use taiko_rs_common::types::UserAction;

pub const USER_ITEM_SIZE:Vector2 = Vector2::new(200.0, 100.0);

#[derive(Clone)]
pub struct OnlineUser {
    pos: Vector2,
    hover: bool,
    selected: bool,

    pub user_id: u32,
    pub username: String,
    pub action: Option<UserAction>,
    pub action_text: Option<String>
}
impl OnlineUser {
    pub fn new(user_id:u32, username:String) -> Self {
        Self {
            user_id,
            username,
            action:None,
            action_text: None,

            hover: false,
            selected: false,
            pos: Vector2::zero()
        }
    }
}

impl ScrollableItem for OnlineUser {
    fn size(&self) -> Vector2 {USER_ITEM_SIZE}
    fn get_pos(&self) -> Vector2 {Vector2::zero()}
    fn set_pos(&mut self, pos:Vector2) {self.pos = pos}
    fn get_tag(&self) -> String {self.username.clone()}
    fn set_tag(&mut self, _tag:&str) {}

    fn get_hover(&self) -> bool {self.hover}
    fn set_hover(&mut self, hover:bool) {self.hover = hover}
    fn get_selected(&self) -> bool {self.selected}
    fn set_selected(&mut self, selected:bool) {self.selected = selected}


    fn draw(&mut self, _args:piston::RenderArgs, pos_offset:Vector2, depth:f64) -> Vec<Box<dyn Renderable>> {
        let mut list:Vec<Box<dyn Renderable>> = Vec::new();
        let font = get_font("main");

        // bounding box
        let mut c = Color::BLACK.clone();
        c.a = 0.5;
        list.push(Box::new(Rectangle::new(
            c,
            depth,
            pos_offset,
            USER_ITEM_SIZE,
            Some(Border::new(if self.hover {Color::RED} else {Color::BLACK}, 2.0))
        )));

        // username
        list.push(Box::new(Text::new(
            Color::BLACK,
            depth - 1.0,
            pos_offset + Vector2::new(5.0, 20.0),
            20,
            self.username.clone(),
            font.clone()
        )));

        // status
        if let Some(_action) = &self.action {

        }
        if let Some(action_text) = &self.action_text {
            list.push(Box::new(Text::new(
                Color::BLACK,
                depth - 1.0,
                pos_offset + Vector2::new(5.0, 50.0),
                20,
                action_text.clone(),
                font.clone()
            )));
        }

        list
    }

    fn on_click(&mut self, _pos:Vector2, _button:piston::MouseButton, _mods: KeyModifiers) -> bool {
        self.hover
    }

}