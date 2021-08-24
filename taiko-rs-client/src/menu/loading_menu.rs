use std::{fs::read_dir, sync::Arc, time::Duration};

use tokio::time::sleep;
use parking_lot::Mutex;

use crate::game::Audio;
use crate::render::{Color, Rectangle, Text};
use crate::{SONGS_DIR, WINDOW_SIZE, Vector2, menu::Menu};
use crate::game::{Game, helpers::BeatmapManager};
use taiko_rs_common::{types::Score, serialization::Serializable};

/// helper for when starting the game. will load beatmaps, settings, etc from storage
/// all while providing the user with its progress (relatively anyways)
pub struct LoadingMenu {
    pub complete: bool,
    status: Arc<Mutex<LoadingStatus>>,

    beatmap_manager:Arc<Mutex<BeatmapManager>>
}

impl LoadingMenu {
    pub fn new(beatmap_manager: Arc<Mutex<BeatmapManager>>) -> Self {
        Self {
            complete: false,
            beatmap_manager: beatmap_manager.clone(),
            status: Arc::new(Mutex::new(LoadingStatus::new(beatmap_manager)))
        }
    }
    pub fn load(&mut self, game:&Game) {
        let status = self.status.clone();
        
        game.threading.spawn(async move {
            let status = status.clone();

            // preload audio 
            Self::load_audio(status.clone()).await;

            // load beatmaps
            Self::load_beatmaps(status.clone()).await;
            
            // load scores
            Self::load_scores(status.clone()).await;

            status.lock().stage = LoadingStage::Done;
        });
    }

    // loaders
    async fn load_audio(status: Arc<Mutex<LoadingStatus>>) {
        status.lock().stage = LoadingStage::Audio;
        // get a value from the hash, will do the lazy_static stuff and populate
        let a = Audio::play_preloaded("don").upgrade();
        a.unwrap().stop();
    }

    async fn load_beatmaps(status: Arc<Mutex<LoadingStatus>>) {
        status.lock().stage = LoadingStage::Beatmaps;
        // set the count and reset the counter
        status.lock().loading_count = 0;
        status.lock().loading_done = 0;

        let mut folders = Vec::new();
        read_dir(SONGS_DIR)
            .unwrap()
            .for_each(|f| {
                let f = f.unwrap().path();
                folders.push(f.to_str().unwrap().to_owned());
            });
        status.lock().loading_count = folders.len();

        for f in folders {
            status.lock().beatmap_manager.lock().check_folder(f);
            status.lock().loading_done += 1;
        }
    }

    async fn load_scores(status: Arc<Mutex<LoadingStatus>>) {
        status.lock().stage = LoadingStage::Scores;

        // set the count and reset the counter
        status.lock().loading_count = 0;
        status.lock().loading_done = 0;

        let reader = taiko_rs_common::serialization::open_database(crate::SCORE_DATABASE_FILE);
        match reader {
            Err(e) => {
                println!("Error reading scores db: {:?}", e);

                status.lock().error = Some("Error reading scores db".to_owned());
                sleep(Duration::from_secs(1)).await;
                status.lock().error = None;
            }
            Ok(mut reader) => {
                let count = reader.read_u128();
                status.lock().loading_count = count as usize;
                
                for _ in 0..count {
                    let score = Score::read(&mut reader);
                    crate::databases::save_score(&score);
                    status.lock().loading_done += 1;
                }
            }
        }
    }
}

impl Menu<Game> for LoadingMenu {
    fn update(&mut self, game:&mut Game) {
        if let LoadingStage::Done = self.status.lock().stage {
            let menu = game.menus.get("main").unwrap().clone();
            game.queue_state_change(crate::game::GameState::InMenu(menu));

            // select a map to load bg and intro audio from (TODO! add our own?)
            let mut manager = self.beatmap_manager.lock();

            if let Some(map) = manager.random_beatmap() {
                manager.set_current_beatmap(game, map);
            }
            
        }
    }

    fn draw(&mut self, _args:piston::RenderArgs) -> Vec<Box<dyn crate::render::Renderable>> {
        let mut list: Vec<Box<dyn crate::render::Renderable>> = Vec::new();
        let font = crate::game::get_font("main");

        // since this is just loading, we dont care about performance here
        let state = self.status.lock();

        let mut text:Text;
        match &state.error {
            Some(error) => {
                text = Text::new(
                    Color::BLACK,
                    -100.0,
                    Vector2::zero(),
                    32,
                    error.clone(),
                    font
                )
            },
            None => match state.stage {
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
                LoadingStage::Audio => {
                    text = Text::new(
                        Color::BLACK,
                        -100.0,
                        Vector2::zero(),
                        32,
                        format!("Loading Audio"),
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
    error: Option<String>,
    beatmap_manager: Arc<Mutex<BeatmapManager>>,

    loading_count: usize, // items in the list
    loading_done: usize // items done loading in the list
}
impl LoadingStatus {
    pub fn new(beatmap_manager: Arc<Mutex<BeatmapManager>>) -> Self {
        Self {
            beatmap_manager,
            error: None,
            loading_count: 0,
            loading_done: 0,
            stage: LoadingStage::None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum LoadingStage {
    None,
    Beatmaps,
    Scores,
    Audio,

    Done,
}