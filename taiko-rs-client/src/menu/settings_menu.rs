use std::sync::{Arc, Mutex};

use piston::{MouseButton, RenderArgs};

use crate::render::*;
use crate::game::{Game, GameMode, KeyModifiers, Settings, Vector2};
use crate::menu::{Menu, TextInput, MenuButton, KeyButton, PasswordInput, ScrollableArea, ScrollableItem, Checkbox, Slider};

const BUTTON_SIZE:Vector2 = Vector2::new(100.0, 50.0);
const KEYBUTTON_SIZE:Vector2 = Vector2::new(400.0, 50.0);

pub struct SettingsMenu {
    scroll_area: ScrollableArea,
}
impl SettingsMenu {
    pub fn new() -> SettingsMenu {
        let settings = Settings::get();
        let p = Vector2::new(10.0, 0.0); // scroll area edits the y

        // setup items
        let mut scroll_area = ScrollableArea::new(p, Vector2::new(500.0, 500.0), true);
        let mut username_input = TextInput::new(p, Vector2::new(600.0, 50.0), "Username", &settings.username);
        let mut password_input = PasswordInput::new(p, Vector2::new(600.0, 50.0), "Password", &settings.password);
        let mut done_button =  MenuButton::new(p, BUTTON_SIZE, "Done");
        let mut left_kat_btn = KeyButton::new(p, KEYBUTTON_SIZE, settings.left_kat, "Left Kat");
        let mut left_don_btn = KeyButton::new(p, KEYBUTTON_SIZE, settings.left_don, "Left Don");
        let mut right_don_btn = KeyButton::new(p, KEYBUTTON_SIZE, settings.right_don, "Right Don");
        let mut right_kat_btn = KeyButton::new(p, KEYBUTTON_SIZE, settings.right_kat, "Right Kat");
        let mut static_sv = Checkbox::new(p, Vector2::new(200.0, BUTTON_SIZE.y), "No Sv Changes", settings.static_sv);
        let mut sv_mult = Slider::new(p, Vector2::new(400.0, BUTTON_SIZE.y), "Slider Multiplier", settings.sv_multiplier as f64, Some(0.1..2.0), None);

        // add tags
        username_input.set_tag("username");
        password_input.set_tag("password");
        done_button.set_tag("done");
        left_kat_btn.set_tag("left_kat");
        left_don_btn.set_tag("left_don");
        right_don_btn.set_tag("right_don");
        right_kat_btn.set_tag("right_kat");
        static_sv.set_tag("static_sv");
        sv_mult.set_tag("sv_mult");

        // add to scroll area
        scroll_area.add_item(Box::new(username_input));
        scroll_area.add_item(Box::new(password_input));
        scroll_area.add_item(Box::new(left_kat_btn));
        scroll_area.add_item(Box::new(left_don_btn));
        scroll_area.add_item(Box::new(right_don_btn));
        scroll_area.add_item(Box::new(right_kat_btn));
        scroll_area.add_item(Box::new(static_sv));
        // scroll_area.add_item(Box::new(sv_mult)); // broken

        scroll_area.add_item(Box::new(done_button));

        SettingsMenu {
            scroll_area
        }
    }

    pub fn finalize(&self, game:Arc<Mutex<&mut Game>>) {
        // write settings to settings
        let mut settings = Settings::get_mut();

        //TODO: can we setup a macro for this?
        if let Some(username) = self.scroll_area.get_tagged("username".to_owned()).first().unwrap().get_value().downcast_ref::<String>() {
            // println!("username => {}", username);
            settings.username = username.to_owned();
        }
        if let Some(password) = self.scroll_area.get_tagged("password".to_owned()).first().unwrap().get_value().downcast_ref::<String>() {
            // println!("password => {}", password);
            settings.password = password.to_owned();
        }

        if let Some(key) = self.scroll_area.get_tagged("left_kat".to_owned()).first().unwrap().get_value().downcast_ref::<piston::Key>() {
            // println!("lk => {:?}", key);
            settings.left_kat = key.clone();
        }
        if let Some(key) = self.scroll_area.get_tagged("left_don".to_owned()).first().unwrap().get_value().downcast_ref::<piston::Key>() {
            // println!("ld => {:?}", key);
            settings.left_don = key.clone();
        } 
        if let Some(key) = self.scroll_area.get_tagged("right_don".to_owned()).first().unwrap().get_value().downcast_ref::<piston::Key>() {
            // println!("rd => {:?}", key);
            settings.right_don = key.clone();
        }
        if let Some(key) = self.scroll_area.get_tagged("right_kat".to_owned()).first().unwrap().get_value().downcast_ref::<piston::Key>() {
            // println!("rk => {:?}", key);
            settings.right_kat = key.clone();
        }

        if let Some(val) = self.scroll_area.get_tagged("static_sv".to_owned()).first().unwrap().get_value().downcast_ref::<bool>() {
            // println!("rk => {:?}", key);
            settings.static_sv = val.clone();
        }
        // if let Some(val) = self.scroll_area.get_tagged("sv_mult".to_owned()).first().unwrap().get_value().downcast_ref::<f64>() {
        //     settings.sv_multiplier = val.clone() as f32;
        // }
        settings.save();

        let mut game = game.lock().unwrap();
        let menu = game.menus.get("main").unwrap().clone();
        game.queue_mode_change(GameMode::InMenu(menu));
    }
}
impl Menu for SettingsMenu {
    fn update(&mut self, _game:Arc<Mutex<&mut Game>>) {
        self.scroll_area.update();
    }

    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        // let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        // list.extend(self.scroll_area.draw(args));
        // list

        self.scroll_area.draw(args)
    }

    fn on_click(&mut self, pos:Vector2, button:MouseButton, game:Arc<Mutex<&mut Game>>) {
        if let Some(tag) = self.scroll_area.on_click(pos, button, game.clone()) {
            match tag.as_str() {
                "done" => self.finalize(game.clone()),
                _ => {}
            }
        }
    }

    fn on_mouse_move(&mut self, pos:Vector2, game:Arc<Mutex<&mut Game>>) {
        self.scroll_area.on_mouse_move(pos, game);
    }

    fn on_key_press(&mut self, key:piston::Key, game:Arc<Mutex<&mut Game>>, mods:KeyModifiers) {
        self.scroll_area.on_key_press(key, mods);

        if key == piston::Key::Escape {
            let mut game = game.lock().unwrap();
            let menu = game.menus.get("main").unwrap().clone();
            game.queue_mode_change(GameMode::InMenu(menu));
            return;
        }
    }

    fn on_text(&mut self, text:String) {
        self.scroll_area.on_text(text);
    }
}
