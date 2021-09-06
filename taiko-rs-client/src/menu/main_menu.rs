use std::sync::Arc;

use parking_lot::Mutex;
use piston::{MouseButton, RenderArgs};
use ayyeve_piston_ui::menu::KeyModifiers;

use crate::{window_size, Vector2, render::*};
use crate::visualization::{MenuVisualization, Visualization};
use crate::menu::{Menu, MenuButton, OsuDirectMenu, ScrollableItem};
use crate::game::{Audio, Game, GameState, get_font, managers::BEATMAP_MANAGER};

const BUTTON_SIZE: Vector2 = Vector2::new(100.0, 50.0);
const Y_MARGIN: f64 = 20.0;
const Y_OFFSET: f64 = 10.0;

pub struct MainMenu {
    pub play_button: MenuButton,
    pub direct_button: MenuButton,
    pub settings_button: MenuButton,
    pub exit_button: MenuButton,

    visualization: MenuVisualization
}
impl MainMenu {
    pub fn new() -> MainMenu {
        let middle = window_size().x /2.0 - BUTTON_SIZE.x/2.0;
        let mut counter = 1.0;
        
        let play_button = MenuButton::new(Vector2::new(middle, (BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET), BUTTON_SIZE, "Play");
        counter += 1.0;
        let direct_button = MenuButton::new(Vector2::new(middle, (BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET), BUTTON_SIZE, "osu!Direct");
        counter += 1.0;
        let settings_button = MenuButton::new(Vector2::new(middle, (BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET), BUTTON_SIZE, "Settings");
        counter += 1.0;
        let exit_button = MenuButton::new(Vector2::new(middle, (BUTTON_SIZE.y + Y_MARGIN) * counter + Y_OFFSET), BUTTON_SIZE, "Exit");

        MainMenu {
            play_button,
            direct_button,
            settings_button,
            exit_button,

            visualization: MenuVisualization::new()
        }
    }
}
impl Menu<Game> for MainMenu {
    fn on_change(&mut self, _into:bool) {
        self.visualization.reset();
    }

    fn update(&mut self, g:&mut Game) {
        if let None = Audio::get_song() {
            println!("song done");
            let map = BEATMAP_MANAGER.lock().random_beatmap();

            // it should?
            if let Some(map) = map {
                BEATMAP_MANAGER.lock().set_current_beatmap(g, &map, false, false);
            }
        }

        let maps = BEATMAP_MANAGER.lock().get_new_maps();
        if maps.len() > 0 {
            BEATMAP_MANAGER.lock().set_current_beatmap(g, &maps[maps.len() - 1], true, false);
        }
    }

    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        let pos_offset = Vector2::zero();
        let depth = 0.0;

        // draw welcome text
        let mut welcome_text = Text::new(
            Color::BLACK,
            depth-1.0,
            pos_offset,
            40,
            "Welcome to Taiko.rs".to_owned(),
            get_font("main")
        );
        welcome_text.center_text(Rectangle::bounds_only(Vector2::new(0.0, 30.0), Vector2::new(window_size().x , 50.0)));
        
        list.push(crate::helpers::visibility_bg(welcome_text.pos - Vector2::new(0.0, 40.0), Vector2::new(welcome_text.measure_text().x , 50.0)));
        list.push(Box::new(welcome_text));

        // draw buttons
        list.extend(self.play_button.draw(args, pos_offset, depth));
        list.extend(self.direct_button.draw(args, pos_offset, depth));
        list.extend(self.settings_button.draw(args, pos_offset, depth));
        list.extend(self.exit_button.draw(args, pos_offset, depth));

        // visualization
        let mid = window_size() / 2.0;
        self.visualization.draw(args, mid, depth + 10.0, &mut list);


        // slider drawing testing
        
        // use crate::gameplay::Beatmap;
        // {
        //     let mut map = Beatmap {
        //         hash: "".to_owned(),
        //         metadata: crate::gameplay::BeatmapMeta::new(),
        //         timing_points: vec![
        //             crate::gameplay::TimingPoint {
        //                 time: 0.0,
        //                 beat_length: 100.0,
        //                 volume: 0,
        //                 kiai: true
        //             }
        //         ],
        //         notes: Vec::new(),
        //         sliders: Vec::new(),
        //         spinners: Vec::new(),
        //         holds: Vec::new(),
        //     };
        //     map.metadata.beatmap_version = 9.0;

        //     const N:f64 = 50.0;

        //     let slider = crate::gameplay::hitobject_defs::SliderDef {
        //         pos: Vector2::new(168.0, 208.0),
        //         time: 0.0,
        //         hitsound: 0,
        //         curve_type: crate::gameplay::hitobject_defs::CurveType::Bézier,
        //         curve_points: vec![
        //             Vector2::new(240.0, 248.0),
        //             Vector2::new(320.0, 192.0),
        //             Vector2::new(296.0, 104.0),
        //         ],
        //         slides: 1,
        //         length: 0.0,
        //         edge_sets: vec![],
        //         edge_sounds: vec![],
        //         hitsamples: vec![],
        //         raw_str: "".to_owned()
        //     };

        //     let curve = crate::helpers::slider::get_curve(&slider, &map);

        //     let pos_offset = window_size() / 2.0;

        //     for i in 0..curve.path.len() {
        //         let color = [
        //             Color::GREEN,
        //             Color::RED
        //         ][i % 2];
        //         let depth = -1000.0;
        //         let radius = 20.0;
    
        //         let line = curve.path[i];
        //         list.push(Box::new(Line::new(
        //             pos_offset + line.p1,
        //             pos_offset + line.p2,
        //             radius,
        //             depth,
        //             color
        //         )));
        //     }

        // }




        list
    }

    fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, game:&mut Game) {
        // switch to beatmap selection
        if self.play_button.on_click(pos, button, mods) {
            let menu = game.menus.get("beatmap").unwrap().clone();
            game.queue_state_change(GameState::InMenu(menu));
            return;
        }

        // open direct menu
        if self.direct_button.on_click(pos, button, mods) {
            let menu:Arc<Mutex<dyn Menu<Game>>> = Arc::new(Mutex::new(OsuDirectMenu::new()));
            game.queue_state_change(GameState::InMenu(menu));
            return;
        }

        // open settings menu
        if self.settings_button.on_click(pos, button, mods) {
            let menu = game.menus.get("settings").unwrap().clone();
            game.queue_state_change(GameState::InMenu(menu));
            return;
        }

        // quit game
        if self.exit_button.on_click(pos, button, mods) {
            game.queue_state_change(GameState::Closing);
            return;
        }
    }

    fn on_mouse_move(&mut self, pos:Vector2, _game: &mut Game) {
        self.play_button.check_hover(pos);
        self.direct_button.check_hover(pos);
        self.settings_button.check_hover(pos);
        self.exit_button.check_hover(pos);
    }


    fn on_key_press(&mut self, key:piston::Key, game:&mut Game, mods:KeyModifiers) {
        if mods.alt {return}
        use piston::Key::*;
        match key {
            Left => {
                let mut manager = BEATMAP_MANAGER.lock();

                if let Some(map) = manager.previous_beatmap() {
                    manager.set_current_beatmap(game, &map, false, false);
                } else {
                    println!("no prev")
                }
            }
            Right => {
                let mut manager = BEATMAP_MANAGER.lock();

                if let Some(map) = manager.next_beatmap() {
                    manager.set_current_beatmap(game, &map, false, false);
                } else {
                    println!("no next")
                }
            }

            _ => {}
        }
    }
}
