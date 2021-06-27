use std::sync::MutexGuard;
use std::sync::{Arc, Mutex};

use cgmath::Vector2;
use piston::{MouseButton, RenderArgs};

use crate::render::*;
use crate::gameplay::Beatmap;
use crate::game::{Game, GameMode, KeyModifiers};
use crate::menu::{Menu, MenuButton};

use super::ScrollableItem;

const BUTTON_SIZE:Vector2<f64> = Vector2::new(100.0, 50.0);
const Y_MARGIN:f64 = 20.0;
const Y_OFFSET:f64 = 10.0;

pub struct PauseMenu {
    pub beatmap: Arc<Mutex<Beatmap>>,
    pub continue_button: MenuButton,
    pub retry_button: MenuButton,
    pub exit_button: MenuButton
}
impl PauseMenu {
    pub fn new(beatmap:Arc<Mutex<Beatmap>>) -> PauseMenu {
        // beatmap.lock().unwrap().pause();
        PauseMenu {
            beatmap,
            continue_button: MenuButton::new(Vector2::new(10.0,300.0), BUTTON_SIZE, "Continue"),
            retry_button: MenuButton::new(Vector2::new(10.0,300.0), BUTTON_SIZE, "Retry"),
            exit_button: MenuButton::new(Vector2::new(10.0,300.0), BUTTON_SIZE, "Exit"),
        }
    }

    pub fn unpause(&mut self, mut game:MutexGuard<&mut Game>) {
        self.beatmap.lock().unwrap().start();
        game.queue_mode_change(GameMode::Ingame(self.beatmap.clone()));
    }
}
impl Menu for PauseMenu {

    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();

        let window_size = Vector2::new(args.window_size[0], args.window_size[1]);
        let middle_x = window_size.x/2.0 - BUTTON_SIZE.x/2.0;

        // draw buttons
        self.continue_button.pos.y = (BUTTON_SIZE.y + Y_MARGIN) * list.len() as f64 + Y_OFFSET;
        self.continue_button.pos.x = middle_x;
        list.extend(self.continue_button.draw(args, Vector2::new(0.0, 0.0)));

        // self.retry_button.pos.y = args.window_size[1] - (self.retry_button.size.y 0.0);
        self.retry_button.pos.y = (BUTTON_SIZE.y + Y_MARGIN) * list.len() as f64 + Y_OFFSET;
        self.retry_button.pos.x = middle_x;
        list.extend(self.retry_button.draw(args, Vector2::new(0.0, 0.0)));

        // self.retry_button.pos.y = args.window_size[1] - (self.retry_button.size.y 0.0);
        self.exit_button.pos.y = (BUTTON_SIZE.y + Y_MARGIN) * list.len() as f64 + Y_OFFSET;
        self.exit_button.pos.x = middle_x;
        list.extend(self.exit_button.draw(args, Vector2::new(0.0, 0.0)));

        list
    }

    fn on_click(&mut self, _pos:Vector2<f64>, _button:MouseButton, game:Arc<Mutex<&mut Game>>) {
        let mut game = game.lock().unwrap();

        // continue map
        if self.continue_button.hover {
            self.unpause(game);
            return;
        }
        
        // retry
        if self.retry_button.hover {
            self.beatmap.lock().unwrap().reset();
            self.unpause(game);
            return;
        }

        // return to song select
        if self.exit_button.hover {
            let menu = game.menus.get("beatmap").unwrap().to_owned();
            game.queue_mode_change(GameMode::InMenu(menu));
        }
    }

    fn on_mouse_move(&mut self, pos:Vector2<f64>, _game:Arc<Mutex<&mut Game>>) {
        self.continue_button.check_hover(pos);
        self.retry_button.check_hover(pos);
        self.exit_button.check_hover(pos);
    }

    fn on_key_press(&mut self, key:piston::Key, game:Arc<Mutex<&mut Game>>, _mods:KeyModifiers) {
        if key == piston::Key::Escape {
            self.unpause(game.lock().unwrap());
        }
    }
}
