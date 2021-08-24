use std::sync::Arc;

use ayyeve_piston_ui::menu::KeyModifiers;
use ayyeve_piston_ui::menu::menu_elements::Graph;
use parking_lot::Mutex;
use piston::{MouseButton, RenderArgs};


use rustfft::{FftPlanner, num_complex::Complex};

use crate::{WINDOW_SIZE, Vector2, render::*};
use crate::game::{Audio, Game, GameState, get_font};
use crate::menu::{Menu, MenuButton, OsuDirectMenu, ScrollableItem};

const BUTTON_SIZE: Vector2 = Vector2::new(100.0, 50.0);
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
        let middle = WINDOW_SIZE.x /2.0 - BUTTON_SIZE.x/2.0;
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
        }
    }
}
impl Menu<Game> for MainMenu {
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
        welcome_text.center_text(Rectangle::bounds_only(Vector2::new(0.0, 30.0), Vector2::new(WINDOW_SIZE.x , 50.0)));
        
        list.push(crate::helpers::visibility_bg(welcome_text.pos - Vector2::new(0.0, 40.0), Vector2::new(welcome_text.measure_text().x , 50.0)));
        list.push(Box::new(welcome_text));

        // draw buttons
        list.extend(self.play_button.draw(args, pos_offset, depth));
        list.extend(self.direct_button.draw(args, pos_offset, depth));
        list.extend(self.settings_button.draw(args, pos_offset, depth));
        list.extend(self.exit_button.draw(args, pos_offset, depth));


        {
            let audio_data = crate::game::audio::CURRENT_DATA.clone();
            let audio_data = audio_data.lock().clone();
            let len = audio_data.len() - audio_data.len() % 1234;
            let audio_data = &audio_data[0..len];
            let mut audio_data:Vec<Complex<f32>> = audio_data.iter().map(|a| Complex::new(*a, 0.0)).collect();

            let mut planner = FftPlanner::<f32>::new();
            let fft = planner.plan_fft_forward(len / 2);
            fft.process(&mut audio_data);

            // println!("audio_data: {:?}", audio_data);
            let mut graph = Graph::new(
                Vector2::zero(), 
                Vector2::new(500.0, 500.0),
                audio_data.iter().map(|a|a.re).collect(),
                0.0, 1.0
            );
            list.extend(graph.draw(args, pos_offset, depth));
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
}
