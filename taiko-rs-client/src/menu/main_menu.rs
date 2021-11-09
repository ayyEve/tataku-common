use piston::{MouseButton, RenderArgs};
use ayyeve_piston_ui::menu::KeyModifiers;
use taiko_rs_common::types::PlayMode;

use crate::game::managers::ModManager;
use crate::{Vector2, sync::*};
use crate::render::{*, fonts::get_font};
use crate::visualization::{MenuVisualization, Visualization};
use crate::gameplay::{IngameManager, modes::manager_from_playmode};
use crate::menu::{Menu, MenuButton, OsuDirectMenu, ScrollableItem};
use crate::game::{Audio, Game, GameState, Settings, managers::{BEATMAP_MANAGER, NotificationManager}};

const BUTTON_SIZE: Vector2 = Vector2::new(100.0, 50.0);
const Y_MARGIN: f64 = 20.0;
const Y_OFFSET: f64 = 10.0;

pub struct MainMenu {
    pub play_button: MenuButton,
    pub direct_button: MenuButton,
    pub settings_button: MenuButton,
    pub exit_button: MenuButton,

    visualization: MenuVisualization,
    background_game: Option<IngameManager>,
}
impl MainMenu {
    pub fn new() -> MainMenu {
        let middle = Settings::window_size().x /2.0 - BUTTON_SIZE.x/2.0;
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

            visualization: MenuVisualization::new(),
            background_game: None
        }
    }

    fn setup_manager(&mut self) {
        let settings = Settings::get().background_game_settings;
        if !settings.enabled {return}

        let lock = BEATMAP_MANAGER.lock();
        let map = match &lock.current_beatmap {
            Some(map) => map,
            None => return
        };

        match manager_from_playmode(settings.mode, &map) {
            Ok(mut manager) => {
                manager.current_mods = Arc::new(ModManager {
                    autoplay: true,
                    ..Default::default()
                });
                manager.menu_background = true;
                manager.start();

                self.background_game = Some(manager);
            },
            Err(e) => NotificationManager::add_error_notification("Error loading beatmap", e)
        }
    }
}
impl Menu<Game> for MainMenu {
    fn on_change(&mut self, _into:bool) {
        self.visualization.reset();

        self.setup_manager();
    }

    fn update(&mut self, g:&mut Game) {
        if let None = Audio::get_song() {
            println!("song done");
            let map = BEATMAP_MANAGER.lock().random_beatmap();

            // it should?
            if let Some(map) = map {
                BEATMAP_MANAGER.lock().set_current_beatmap(g, &map, false, false);
                self.setup_manager();
            }
        }

        let maps = BEATMAP_MANAGER.lock().get_new_maps();
        if maps.len() > 0 {
            BEATMAP_MANAGER.lock().set_current_beatmap(g, &maps[maps.len() - 1], true, false);
            self.setup_manager();
        }

        self.visualization.update();

        if let Some(manager) = self.background_game.as_mut() {
            manager.update();

            if manager.completed {
                self.background_game = None;
            }
        }
    }

    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        let pos_offset = Vector2::zero();
        let depth = 0.0;
        let window_size = Settings::window_size();

        // draw welcome text
        let mut welcome_text = Text::new(
            Color::BLACK,
            depth-1.0,
            pos_offset,
            40,
            "Welcome to Taiko.rs".to_owned(),
            get_font("main")
        );
        welcome_text.center_text(Rectangle::bounds_only(Vector2::new(0.0, 30.0), Vector2::new(window_size.x , 50.0)));
        
        list.push(crate::helpers::visibility_bg(welcome_text.pos - Vector2::new(0.0, 40.0), Vector2::new(welcome_text.measure_text().x , 50.0)));
        list.push(Box::new(welcome_text));

        // draw buttons
        list.extend(self.play_button.draw(args, pos_offset, depth));
        list.extend(self.direct_button.draw(args, pos_offset, depth));
        list.extend(self.settings_button.draw(args, pos_offset, depth));
        list.extend(self.exit_button.draw(args, pos_offset, depth));

        // visualization
        let mid = window_size / 2.0;
        self.visualization.draw(args, mid, depth + 10.0, &mut list);

        if let Some(manager) = self.background_game.as_mut() {
            manager.draw(args, &mut list);
        }

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
        use piston::Key::*;

        let mut needs_manager_setup = false;


        // check offset keys
        if let Some(manager) = self.background_game.as_mut() {
            let settings = Settings::get();
            if key == settings.key_offset_up {manager.increment_offset(5.0)}
            if key == settings.key_offset_down {manager.increment_offset(-5.0)}
        }


        if !mods.alt {
            match key {
                Left => {
                    let mut manager = BEATMAP_MANAGER.lock();

                    if let Some(map) = manager.previous_beatmap() {
                        manager.set_current_beatmap(game, &map, false, false);
                        needs_manager_setup = true;
                    } else {
                        println!("no prev")
                    }
                }
                Right => {
                    let mut manager = BEATMAP_MANAGER.lock();

                    if let Some(map) = manager.next_beatmap() {
                        manager.set_current_beatmap(game, &map, false, false);
                        needs_manager_setup = true;
                    } else {
                        println!("no next")
                    }
                }

                _ => {}
            }
        }
        
        if mods.alt {
            let new_mode = match key {
                D1 => Some(PlayMode::Standard),
                D2 => Some(PlayMode::Taiko),
                D3 => Some(PlayMode::Catch),
                D4 => Some(PlayMode::Mania),
                _ => None
            };

            if let Some(new_mode) = new_mode {
                let mut settings = Settings::get_mut();
                if settings.background_game_settings.mode != new_mode {
                    needs_manager_setup = true;
                    settings.background_game_settings.mode = new_mode;
                    NotificationManager::add_text_notification(&format!("Menu mode changed to {:?}", new_mode), 1000.0, Color::BLUE);
                }
            }
        }


        if needs_manager_setup {
            self.setup_manager();
        }
    }
}
