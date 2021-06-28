use std::sync::{Arc, Mutex};

use cgmath::Vector2;
use piston::{MouseButton, RenderArgs};

use crate::{WINDOW_SIZE, format};
use crate::render::*;
use crate::gameplay::{Score, HitError};
use crate::game::{Game, GameMode, KeyModifiers, get_font};
use crate::menu::{Menu, MenuButton, ScrollableItem};


const BACK_BUTTON_SIZE:Vector2<f64> = Vector2::new(100.0, 50.0);

pub struct ScoreMenu {
    score: Score,
    back_button: MenuButton,

    // cached
    hit_error:HitError
}
impl ScoreMenu {
    pub fn new(score:Score) -> ScoreMenu {
        let hit_error = score.hit_error();

        ScoreMenu {
            score,
            hit_error,
            back_button: MenuButton::new(Vector2::new(10.0,WINDOW_SIZE.y as f64 - BACK_BUTTON_SIZE.y + 10.0), BACK_BUTTON_SIZE, "Back")
        }
    }
}
impl Menu for ScoreMenu {
    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        let font = get_font("main");

        // draw score info
        let score_txt = Text::new(
            Color::BLACK,
            1.0,
            Vector2::new(50.0, 100.0),
            30,
            format!("Score: {}", format(self.score.score)),
            font.clone()
        );
        list.push(Box::new(score_txt));

        // counts
        let x300_txt = Text::new(
            Color::BLACK,
            1.0,
            Vector2::new(50.0, 140.0),
            30,
            format!("x300: {}", format(self.score.x300)),
            font.clone()
        );
        list.push(Box::new(x300_txt));
        let x100_txt = Text::new(
            Color::BLACK,
            1.0,
            Vector2::new(50.0, 170.0),
            30,
            format!("x100: {}", format(self.score.x100)),
            font.clone()
        );
        list.push(Box::new(x100_txt));
        let miss_txt = Text::new(
            Color::BLACK,
            1.0,
            Vector2::new(50.0, 200.0),
            30,
            format!("Miss: {}", format(self.score.xmiss)),
            font.clone()
        );
        list.push(Box::new(miss_txt));

        // combo and acc
        let combo_txt = Text::new(
            Color::BLACK,
            1.0,
            Vector2::new(50.0, 240.0),
            30,
            format!("{}x, {:.2}%", format(self.score.max_combo), self.score.acc() * 100.0),
            font.clone()
        );
        list.push(Box::new(combo_txt));

        let error_txt = Text::new(
            Color::BLACK,
            1.0,
            Vector2::new(50.0, 280.0),
            30,
            format!("Error: {:.2}ms - {:.2}ms avg", self.hit_error.early, self.hit_error.late),
            font.clone()
        );
        list.push(Box::new(error_txt));
        let error2_txt = Text::new(
            Color::BLACK,
            1.0,
            Vector2::new(50.0, 320.0),
            30,
            format!("Unstable Rate: {:.2}", self.hit_error.unstable_rate),
            font.clone()
        );
        list.push(Box::new(error2_txt));
        

        // draw buttons
        list.extend(self.back_button.draw(args, Vector2::new(0.0, 0.0)));

        list
    }

    fn on_click(&mut self, pos:Vector2<f64>, button:MouseButton, game:Arc<Mutex<&mut Game>>) {
        // check if back button was clicked
        if self.back_button.on_click(pos, button) {
            let mut game = game.lock().unwrap();
            let menu = game.menus.get("beatmap").unwrap().to_owned();
            game.queue_mode_change(GameMode::InMenu(menu));
        }
    }

    fn on_mouse_move(&mut self, pos:Vector2<f64>, _game:Arc<Mutex<&mut Game>>) {
        self.back_button.on_mouse_move(pos);
    }

    fn on_key_press(&mut self, key:piston::Key, game:Arc<Mutex<&mut Game>>, _mods:KeyModifiers) {
        if key == piston::Key::Escape {
            let mut game = game.lock().unwrap();
            game.current_mode = GameMode::InMenu(game.menus.get("beatmap").unwrap().to_owned());
        }
    }
}
