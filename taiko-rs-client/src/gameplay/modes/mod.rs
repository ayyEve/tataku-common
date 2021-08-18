use taiko_rs_common::types::PlayMode;

use self::taiko::TaikoGame;
use super::{Beatmap, GameMode};

pub mod taiko;


use PlayMode::*;
pub fn select_gamemode_from_playmode(playmode:PlayMode, beatmap:&Beatmap) -> impl GameMode {
    match playmode {
        Standard => {
            TaikoGame::new(beatmap)
        },
        Taiko => {
            TaikoGame::new(beatmap)
        },
        Catch => {
            TaikoGame::new(beatmap)
        },
        Mania => {
            TaikoGame::new(beatmap)
        },
    }
}