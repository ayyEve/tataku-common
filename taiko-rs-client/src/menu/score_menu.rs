use std::sync::Arc;

use piston::{MouseButton, RenderArgs};
use parking_lot::Mutex;

use crate::gameplay::Beatmap;
use taiko_rs_common::types::{Score, HitError};
use crate::{WINDOW_SIZE, databases, format, render::*};
use crate::menu::{Menu, MenuButton, ScrollableItem, Graph};
use crate::game::{Game, GameMode, KeyModifiers, get_font, Vector2};

const GRAPH_SIZE:Vector2 = Vector2::new(400.0, 200.0);
const GRAPH_PADDING:Vector2 = Vector2::new(10.0,10.0);

pub struct ScoreMenu {
    score: Score,
    beatmap: Arc<Mutex<Beatmap>>,
    back_button: MenuButton,
    replay_button: MenuButton,
    graph: Graph,

    // cached
    hit_error: HitError
}
impl ScoreMenu {
    pub fn new(score:&Score, beatmap: Arc<Mutex<Beatmap>>) -> ScoreMenu {
        let hit_error = score.hit_error();
        let back_button = MenuButton::back_button();

        let graph = Graph::new(
            WINDOW_SIZE - (GRAPH_SIZE + GRAPH_PADDING),
            GRAPH_SIZE,
            score.hit_timings.iter().map(|e|*e as f32).collect(),
            -50.0,
            50.0
        );

        ScoreMenu {
            score: score.clone(),
            beatmap,
            hit_error,
            graph,
            replay_button: MenuButton::new(back_button.get_pos() - Vector2::new(0.0, back_button.size().y+5.0), back_button.size(), "Replay"),
            back_button,
        }
    }
}
impl Menu for ScoreMenu {
    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        let font = get_font("main");

        let depth = 0.0;

        // graph
        list.extend(self.graph.draw(args, Vector2::zero(), depth));

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
            format!("Deviance: {:.2}ms", self.hit_error.deviance),
            font.clone()
        )));
        
        // draw buttons
        list.extend(self.back_button.draw(args, Vector2::zero(), depth));
        list.extend(self.replay_button.draw(args, Vector2::zero(), depth));

        list
    }

    fn on_click(&mut self, pos:Vector2, button:MouseButton, game:&mut Game) {
        if self.replay_button.on_click(pos, button) {
            self.beatmap.lock().reset();

            let replay = databases::get_local_replay(self.score.hash());

            match replay {
                Ok(replay) => {
                    game.menus.get("beatmap").unwrap().lock().on_change(false);
                    game.queue_mode_change(GameMode::Replaying(self.beatmap.clone(), replay.clone(), 0));
                },
                Err(e) => println!("error loading replay: {}", e),
            }
        }

        if self.back_button.on_click(pos, button) {
            let menu = game.menus.get("beatmap").unwrap().to_owned();
            game.queue_mode_change(GameMode::InMenu(menu));
        }
    }

    fn on_mouse_move(&mut self, pos:Vector2, _game:&mut Game) {
        self.replay_button.on_mouse_move(pos);
        self.back_button.on_mouse_move(pos);
        self.graph.on_mouse_move(pos);
    }

    fn on_key_press(&mut self, key:piston::Key, game: &mut Game, _mods:KeyModifiers) {
        if key == piston::Key::Escape {
            game.current_mode = GameMode::InMenu(game.menus.get("beatmap").unwrap().to_owned());
        }
    }
}
