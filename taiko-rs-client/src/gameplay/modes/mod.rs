use std::sync::Arc;

use parking_lot::Mutex;
use taiko_rs_common::types::PlayMode;

use super::{Beatmap, GameMode};

pub mod taiko;
pub mod mania;
pub mod catch;
pub mod standard;

use PlayMode::*;
pub fn select_gamemode_from_playmode(mut playmode: PlayMode, beatmap: &Beatmap) -> Arc<Mutex<dyn GameMode>> {
    // println!("playmode: {:?}", playmode);
    if beatmap.metadata.mode != Standard {
        playmode = beatmap.metadata.mode;
    }

    match playmode {
        Standard => Arc::new(Mutex::new(standard::StandardGame::new(beatmap))),
        Taiko => Arc::new(Mutex::new(taiko::TaikoGame::new(beatmap))),
        Catch => Arc::new(Mutex::new(catch::CatchGame::new(beatmap))),
        Mania => Arc::new(Mutex::new(mania::ManiaGame::new(beatmap)))
    }
}