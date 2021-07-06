use std::sync::{Arc, Mutex};

use cgmath::Vector2;
use piston::{MouseButton, RenderArgs};

use crate::{format, render::*};
use crate::gameplay::{Beatmap, HitError, Score};
use crate::game::{Game, GameMode, KeyModifiers, get_font};
use crate::menu::{Menu, MenuButton, ScrollableItem};

pub struct ScoreMenu {
    score: Score,
    beatmap:Arc<Mutex<Beatmap>>,
    back_button: MenuButton,
    replay_button: MenuButton,

    // cached
    hit_error:HitError
}
impl ScoreMenu {
    pub fn new(score:Score, beatmap: Arc<Mutex<Beatmap>>) -> ScoreMenu {
        let hit_error = score.hit_error();
        let back_button = MenuButton::back_button();

        ScoreMenu {
            score,
            beatmap,
            hit_error,
            replay_button: MenuButton::new(back_button.get_pos() - Vector2::new(0.0, back_button.size().y), back_button.size(), "Replay"),
            back_button,
        }
    }
}
impl Menu for ScoreMenu {
    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        let font = get_font("main");

        let depth = 0.0;

        // draw score info
        list.push(Box::new(Text::new(
            Color::BLACK,
            depth + 1.0,
            Vector2::new(50.0, 100.0),
            30,
            format!("Score: {}", format(self.score.score)),
            font.clone()
        )));

        // counts
        list.push(Box::new(Text::new(
            Color::BLACK,
            depth + 1.0,
            Vector2::new(50.0, 140.0),
            30,
            format!("x300: {}", format(self.score.x300)),
            font.clone()
        )));
        list.push(Box::new(Text::new(
            Color::BLACK,
            depth + 1.0,
            Vector2::new(50.0, 170.0),
            30,
            format!("x100: {}", format(self.score.x100)),
            font.clone()
        )));
        list.push(Box::new(Text::new(
            Color::BLACK,
            depth + 1.0,
            Vector2::new(50.0, 200.0),
            30,
            format!("Miss: {}", format(self.score.xmiss)),
            font.clone()
        )));

        // combo and acc
        list.push(Box::new(Text::new(
            Color::BLACK,
            depth + 1.0,
            Vector2::new(50.0, 240.0),
            30,
            format!("{}x, {:.2}%", format(self.score.max_combo), self.score.acc() * 100.0),
            font.clone()
        )));

        list.push(Box::new(Text::new(
            Color::BLACK,
            depth + 1.0,
            Vector2::new(50.0, 280.0),
            30,
            format!("Error: {:.2}ms - {:.2}ms avg", self.hit_error.early, self.hit_error.late),
            font.clone()
        )));
        list.push(Box::new(Text::new(
            Color::BLACK,
            depth + 1.0,
            Vector2::new(50.0, 320.0),
            30,
            format!("Unstable Rate: {:.2}", self.hit_error.unstable_rate),
            font.clone()
        )));
        

        // draw buttons
        list.extend(self.back_button.draw(args, Vector2::new(0.0, 0.0), depth));
        list.extend(self.replay_button.draw(args, Vector2::new(0.0, 0.0), depth));

        list
    }

    fn on_click(&mut self, pos:Vector2<f64>, button:MouseButton, game:Arc<Mutex<&mut Game>>) {
        if self.replay_button.on_click(pos, button) {
            self.beatmap.lock().unwrap().reset();
            let mut game = game.lock().unwrap();

            game.menus.get("beatmap").unwrap().lock().unwrap().on_change(false);
            game.queue_mode_change(GameMode::Replaying(self.beatmap.clone(), self.score.replay.clone(), 0));
        }

        if self.back_button.on_click(pos, button) {
            let mut game = game.lock().unwrap();
            let menu = game.menus.get("beatmap").unwrap().to_owned();
            game.queue_mode_change(GameMode::InMenu(menu));
        }
    }

    fn on_mouse_move(&mut self, pos:Vector2<f64>, _game:Arc<Mutex<&mut Game>>) {
        self.replay_button.on_mouse_move(pos);
        self.back_button.on_mouse_move(pos);
    }

    fn on_key_press(&mut self, key:piston::Key, game:Arc<Mutex<&mut Game>>, _mods:KeyModifiers) {
        if key == piston::Key::Escape {
            let mut game = game.lock().unwrap();
            game.current_mode = GameMode::InMenu(game.menus.get("beatmap").unwrap().to_owned());
        }
    }
}
