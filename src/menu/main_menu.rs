use std::sync::{Arc, Mutex};

use cgmath::Vector2;
use piston::{MouseButton, RenderArgs};

use crate::render::*;
use crate::WINDOW_SIZE;
use crate::game::{Game, GameMode, get_font};
use crate::menu::{Menu, MenuButton, OsuDirectMenu, ScrollableItem};

const BUTTON_SIZE: Vector2<f64> = Vector2::new(100.0, 50.0);
const Y_MARGIN: f64 = 20.0;
const Y_OFFSET: f64 = 10.0;

pub struct MainMenu {
    pub play_button: MenuButton,
    pub direct_button: MenuButton,
    pub settings_button: MenuButton,
    pub exit_button: MenuButton
}
impl MainMenu {
    pub fn new() -> MainMenu {
        let middle = WINDOW_SIZE.x as f64/2.0 - BUTTON_SIZE.x/2.0;
        let mut counter = 1.0;
        
        let play_button = MenuButton::new(Vector2::new(middle,(BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET), BUTTON_SIZE, "Play");
        counter += 1.0;
        let direct_button = MenuButton::new(Vector2::new(middle,(BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET), BUTTON_SIZE, "osu!Direct");
        counter += 1.0;
        let settings_button = MenuButton::new(Vector2::new(middle,(BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET), BUTTON_SIZE, "Settings");
        counter += 1.0;
        let exit_button = MenuButton::new(Vector2::new(middle,(BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET), BUTTON_SIZE, "Exit");


        MainMenu {
            play_button,
            direct_button,
            settings_button,
            exit_button,
        }
    }
}
impl Menu for MainMenu {
    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();

        // let mut counter = 0.0;
        let window_size = Vector2::new(args.window_size[0], args.window_size[1]);

        // draw welcome text
        let welcome_rect = Rectangle::new(
            Color::BLACK,
            0.0,
            Vector2::new(0.0, 30.0),
            Vector2::new(window_size.x, 50.0),
            None
        );
        let mut welcome_text = Text::new(
            Color::BLACK,
            -1.0,
            Vector2::new(0.0, 0.0),
            40,
            "Welcome to Taiko.rs".to_owned(),
            get_font("main")
        );
        welcome_text.center_text(welcome_rect);
        list.push(Box::new(welcome_text));
        // counter += 1.0;

        // draw buttons
        // self.play_button.pos.y = (BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET;
        // self.play_button.pos.x = middle_x;
        list.extend(self.play_button.draw(args, Vector2::new(0.0, 0.0)));
        // counter += 1.0;

        // self.direct_button.pos.y = (BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET;
        // self.direct_button.pos.x = middle_x;
        list.extend(self.direct_button.draw(args, Vector2::new(0.0, 0.0)));
        // counter += 1.0;

        // self.settings_button.pos.y = (BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET;
        // self.settings_button.pos.x = middle_x;
        list.extend(self.settings_button.draw(args, Vector2::new(0.0, 0.0)));

        // self.retry_button.pos.y = args.window_size[1] - (self.retry_button.size.y 0.0);
        // self.exit_button.pos.y = (BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET;
        // self.exit_button.pos.x = middle_x;
        list.extend(self.exit_button.draw(args, Vector2::new(0.0, 0.0)));

        list
    }

    fn on_click(&mut self, pos:Vector2<f64>, button:MouseButton, game:Arc<Mutex<&mut Game>>) {
        let mut game = game.lock().unwrap();

        // switch to beatmap selection
        if self.play_button.on_click(pos, button) {
            let menu = game.menus.get("beatmap").unwrap().clone();
            game.queue_mode_change(GameMode::InMenu(menu));
        }

        if self.direct_button.on_click(pos, button) {
            let menu:Arc<Mutex<Box<dyn Menu>>> = Arc::new(Mutex::new(Box::new(OsuDirectMenu::new())));
            game.queue_mode_change(GameMode::InMenu(menu));
        }

        if self.settings_button.on_click(pos, button) {
            let menu = game.menus.get("settings").unwrap().clone();
            game.queue_mode_change(GameMode::InMenu(menu));
        }

        // return to song select
        if self.exit_button.on_click(pos, button) {
            game.queue_mode_change(GameMode::Closing);
            return;
        }
    }

    fn on_mouse_move(&mut self, pos:Vector2<f64>, _game:Arc<Mutex<&mut Game>>) {
        self.play_button.on_mouse_move(pos);
        self.direct_button.on_mouse_move(pos);
        self.settings_button.on_mouse_move(pos);
        self.exit_button.on_mouse_move(pos);
    }
}
