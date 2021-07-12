use std::{fs::read_dir, path::Path, sync::{Arc, Mutex}};

use crate::{SONGS_DIR, WINDOW_SIZE, game::{Game, Settings, Vector2}, gameplay::Beatmap, menu::Menu, render::{Color, Rectangle, Text}};

/// helper for when starting the game. will load beatmaps, settings, etc from storage
/// all while providing the user with its progress (relatively anyways)
pub struct LoadingMenu {
    pub complete: bool,
    status: Arc<Mutex<LoadingStatus>>
}

impl LoadingMenu {
    pub fn new() -> Self {
        Self {
            complete: false,
            status: Arc::new(Mutex::new(LoadingStatus::new()))
        }
    }
    pub fn load(&mut self, game:&Game) {
        let status = self.status.clone();
        
        game.threading.spawn(async move {
            let status = status.clone();

            // load settings (probably pointless, as settings will probably be loaded on game start in the future, if they arent already)
            status.lock().unwrap().stage = LoadingStage::Settings;
            // let settings = Settings::get();
            drop(Settings::get());

            status.lock().unwrap().stage = LoadingStage::Beatmaps;
            // load beatmaps
            {
                let mut folders = Vec::new();
                read_dir(SONGS_DIR).unwrap()
                    .for_each(|f| {
                        let f = f.unwrap().path();
                        folders.push(f.to_str().unwrap().to_owned());
                    });

                // set the count and reset the counter
                {
                    let mut s = status.lock().unwrap();
                    s.loading_count = folders.len();
                    s.loading_done = 0;
                }

                for f in folders {
                    // let f = f.unwrap().path();
                    if !Path::new(&f).is_dir() {continue}
                    let dir_files = read_dir(f).unwrap();

                    for file in dir_files {
                        let file = file.unwrap().path();
                        let file = file.to_str().unwrap();

                        if file.ends_with(".osu") {
                            let map = Beatmap::load(file.to_owned());
                            if map.lock().unwrap().metadata.mode as u8 > 1 {
                                println!("skipping {}, not a taiko map or convert", map.lock().unwrap().metadata.version_string());
                                continue;
                            }
                            status.lock().unwrap().beatmaps.push(map);
                        }
                    }
                }
            }
            
            status.lock().unwrap().stage = LoadingStage::Done;
        });
    }
}

impl Menu for LoadingMenu {

    fn update(&mut self, game:Arc<Mutex<&mut Game>>) {
        if let LoadingStage::Done = self.status.lock().unwrap().stage {
            let mut game = game.lock().unwrap();
            let menu = game.menus.get("main").unwrap().clone();
            game.queue_mode_change(crate::game::GameMode::InMenu(menu));
        }
    }
    

    fn draw(&mut self, _args:piston::RenderArgs) -> Vec<Box<dyn crate::render::Renderable>> {
        let mut list: Vec<Box<dyn crate::render::Renderable>> = Vec::new();
        let font = crate::game::get_font("main");

        // since this is just loading, we dont care about performance here
        let state = self.status.lock().unwrap();

        let mut text:Text;

        match state.stage {
            LoadingStage::None => {
                text = Text::new(
                    Color::BLACK,
                    -100.0,
                    Vector2::zero(),
                    32,
                    format!(""),
                    font
                )
            },
            LoadingStage::Done => {
                text = Text::new(
                    Color::BLACK,
                    -100.0,
                    Vector2::zero(),
                    32,
                    format!("Done"),
                    font
                )
            },
            LoadingStage::Settings => {
                text = Text::new(
                    Color::BLACK,
                    -100.0,
                    Vector2::zero(),
                    32,
                    format!("Loading Settings"),
                    font
                )
            },
            LoadingStage::Beatmaps => {
                text = Text::new(
                    Color::BLACK,
                    -100.0,
                    Vector2::zero(),
                    32,
                    format!("Loading Beatmaps ({}/{})", state.loading_done, state.loading_count),
                    font
                )
            },
            LoadingStage::Scores => {
                text = Text::new(
                    Color::BLACK,
                    -100.0,
                    Vector2::zero(),
                    32,
                    format!("Loading Scores ({}/{})", state.loading_done, state.loading_count),
                    font
                )
            },
        }

        text.center_text(Rectangle::bounds_only(Vector2::zero(), WINDOW_SIZE));
        list.push(Box::new(text));
        list
    }
}

/// async helper
struct LoadingStatus {
    pub stage: LoadingStage,
    pub beatmaps: Vec<Arc<Mutex<Beatmap>>>, // list of beatmaps

    pub loading_count: usize, // items in the list
    pub loading_done: usize // items done loading in the list
}
impl LoadingStatus {
    pub fn new() -> Self {
        Self {
            stage: LoadingStage::None,
            beatmaps: Vec::new(),
            loading_count: 0,
            loading_done: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum LoadingStage {
    None,
    Settings,
    Beatmaps,
    #[allow(dead_code)]
    Scores, // TODO

    Done,
}