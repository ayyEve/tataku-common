use std::{fs::read_dir, sync::Arc};

use parking_lot::Mutex;

use crate::render::{Color, Rectangle, Text};
use crate::{SONGS_DIR, WINDOW_SIZE, menu::Menu};
use crate::game::{Game, Settings, Vector2, helpers::BeatmapManager};

/// helper for when starting the game. will load beatmaps, settings, etc from storage
/// all while providing the user with its progress (relatively anyways)
pub struct LoadingMenu {
    pub complete: bool,
    status: Arc<Mutex<LoadingStatus>>,
}

impl LoadingMenu {
    pub fn new(beatmap_manager: Arc<Mutex<BeatmapManager>>) -> Self {
        Self {
            complete: false,
            status: Arc::new(Mutex::new(LoadingStatus::new(beatmap_manager)))
        }
    }
    pub fn load(&mut self, game:&Game) {
        let status = self.status.clone();
        
        game.threading.spawn(async move {
            let status = status.clone();

            // load settings (probably pointless, as settings will probably be loaded on game start in the future, if they arent already)
            status.lock().stage = LoadingStage::Settings;
            // let settings = Settings::get();
            drop(Settings::get());

            status.lock().stage = LoadingStage::Beatmaps;
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
                    let mut s = status.lock();
                    s.loading_count = folders.len();
                    s.loading_done = 0;
                }

                for f in folders {
                    status.lock().beatmap_manager.lock().check_folder(f);
                    status.lock().loading_done += 1;
                }
            }
            
            status.lock().stage = LoadingStage::Done;
        });
    }
}

impl Menu for LoadingMenu {
    fn update(&mut self, game:&mut Game) {
        if let LoadingStage::Done = self.status.lock().stage {
            let menu = game.menus.get("main").unwrap().clone();
            game.queue_mode_change(crate::game::GameMode::InMenu(menu));
        }
    }

    fn draw(&mut self, _args:piston::RenderArgs) -> Vec<Box<dyn crate::render::Renderable>> {
        let mut list: Vec<Box<dyn crate::render::Renderable>> = Vec::new();
        let font = crate::game::get_font("main");

        // since this is just loading, we dont care about performance here
        let state = self.status.lock();

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
    stage: LoadingStage,
    beatmap_manager: Arc<Mutex<BeatmapManager>>,

    loading_count: usize, // items in the list
    loading_done: usize // items done loading in the list
}
impl LoadingStatus {
    pub fn new(beatmap_manager: Arc<Mutex<BeatmapManager>>) -> Self {
        Self {
            beatmap_manager,
            loading_count: 0,
            loading_done: 0,
            stage: LoadingStage::None,
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