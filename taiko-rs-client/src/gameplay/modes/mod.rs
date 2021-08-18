use std::sync::Arc;

use parking_lot::Mutex;
use taiko_rs_common::types::PlayMode;

use super::{Beatmap, GameMode};

pub mod taiko;
pub mod mania;

use PlayMode::*;
pub fn select_gamemode_from_playmode(playmode:PlayMode, beatmap:&Beatmap) -> Arc<Mutex<dyn GameMode>> {
    println!("playmode: {:?}", playmode);
    match playmode {
        Standard => {
            Arc::new(Mutex::new(taiko::TaikoGame::new(beatmap)))
        }
        Taiko => {
            Arc::new(Mutex::new(taiko::TaikoGame::new(beatmap)))
        }
        Catch => {
            Arc::new(Mutex::new(taiko::TaikoGame::new(beatmap)))
        }
        Mania => {
            Arc::new(Mutex::new(mania::ManiaGame::new(beatmap)))
        }
    }
}