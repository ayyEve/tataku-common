use std::sync::Arc;
use piston::{MouseButton, RenderArgs};

use parking_lot::Mutex;

use crate::game::{Game, GameMode, KeyModifiers};
use crate::menu::{Menu, MenuButton, ScrollableItem};
use crate::{WINDOW_SIZE, Vector2, render::*, gameplay::Beatmap};

const BUTTON_SIZE:Vector2 = Vector2::new(100.0, 50.0);
const Y_MARGIN:f64 = 20.0;
const Y_OFFSET:f64 = 10.0;

pub struct PauseMenu {
    beatmap: Arc<Mutex<Beatmap>>,
    continue_button: MenuButton,
    retry_button: MenuButton,
    exit_button: MenuButton
}
impl PauseMenu {
    pub fn new(beatmap:Arc<Mutex<Beatmap>>) -> PauseMenu {
        let middle = WINDOW_SIZE.x /2.0 - BUTTON_SIZE.x/2.0;

        PauseMenu {
            beatmap,
            continue_button: MenuButton::new(Vector2::new(middle, (BUTTON_SIZE.y + Y_MARGIN) * 0.0 + Y_OFFSET), BUTTON_SIZE, "Continue"),
            retry_button: MenuButton::new(Vector2::new(middle,(BUTTON_SIZE.y + Y_MARGIN) * 1.0 + Y_OFFSET), BUTTON_SIZE, "Retry"),
            exit_button: MenuButton::new(Vector2::new(middle,(BUTTON_SIZE.y + Y_MARGIN) * 2.0 + Y_OFFSET), BUTTON_SIZE, "Exit"),
        }
    }

    pub fn unpause(&mut self, game:&mut Game) {
        self.beatmap.lock().start();
        game.queue_mode_change(GameMode::Ingame(self.beatmap.clone()));
    }
}
impl Menu<Game> for PauseMenu {
    fn get_name(&self) -> &str {"pause"}
    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        let pos_offset = Vector2::zero();
        let depth = 0.0;

        // draw buttons
        list.extend(self.continue_button.draw(args, pos_offset, depth));
        list.extend(self.retry_button.draw(args, pos_offset, depth));
        list.extend(self.exit_button.draw(args, pos_offset, depth));

        list
    }

    fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, game:&mut Game) {
        // continue map
        if self.continue_button.on_click(pos, button, mods) {
            self.unpause(game);
            return;
        }
        
        // retry
        if self.retry_button.on_click(pos, button, mods) {
            self.beatmap.lock().reset();
            self.unpause(game);
            return;
        }

        // return to song select
        if self.exit_button.on_click(pos, button, mods) {
            let menu = game.menus.get("beatmap").unwrap().to_owned();
            game.queue_mode_change(GameMode::InMenu(menu));

            // cleanup memory hogs in the beatmap object
            self.beatmap.lock().cleanup();
        }
    }

    fn on_mouse_move(&mut self, pos:Vector2, _game:&mut Game) {
        self.continue_button.on_mouse_move(pos);
        self.retry_button.on_mouse_move(pos);
        self.exit_button.on_mouse_move(pos);
    }

    fn on_key_press(&mut self, key:piston::Key, game:&mut Game, _mods:KeyModifiers) {
        if key == piston::Key::Escape {
            self.unpause(game);
        }
    }
}
