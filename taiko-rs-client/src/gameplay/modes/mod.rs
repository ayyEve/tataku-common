use std::sync::Arc;

use ayyeve_piston_ui::render::Vector2;
use parking_lot::Mutex;
use taiko_rs_common::types::PlayMode;

use crate::{game::Settings, window_size};

use super::{Beatmap, BeatmapMeta, GameMode, IngameManager};

pub mod taiko;
pub mod mania;
pub mod catch;
pub mod standard;


const FIELD_SIZE:Vector2 = Vector2::new(512.0, 384.0); // 4:3


use PlayMode::*;
pub fn manager_from_playmode(mut playmode: PlayMode, beatmap: &BeatmapMeta) -> IngameManager {
    // println!("playmode: {:?}", playmode);
    if beatmap.mode != Standard {
        playmode = beatmap.mode;
    }

    let beatmap = Beatmap::from_metadata(beatmap);
    let gamemode:Arc<Mutex<dyn GameMode>> = match playmode {
        Standard => Arc::new(Mutex::new(standard::StandardGame::new(&beatmap))),
        Taiko => Arc::new(Mutex::new(taiko::TaikoGame::new(&beatmap))),
        Catch => Arc::new(Mutex::new(catch::CatchGame::new(&beatmap))),
        Mania => Arc::new(Mutex::new(mania::ManiaGame::new(&beatmap)))
    };

    IngameManager::new(beatmap, gamemode)
}


// fn scale_window() -> (f64, Vector2) {
//     let (scale, offset) = Settings::get_mut().standard_settings.get_playfield();
//     let window_size = window_size();
//     let scale = (window_size.y / FIELD_SIZE.y) * scale;

//     let offset = (window_size - FIELD_SIZE * scale) / 2.0 + offset;

//     (scale, offset)
// }

// pub fn scale_coords(osu_coords:Vector2) -> Vector2 {
//     let (scale, offset) = scale_window();
//     offset + osu_coords * scale

//     // osu_coords + Vector2::new((window_size.x - FIELD_SIZE.x) / 2.0, (window_size.y - FIELD_SIZE.y) / 2.0)
// }

// pub fn scale_cs(base:f64) -> f64 {
//     let (scale, _) = scale_window();

//     base * scale
// }


#[derive(Copy, Clone)]
pub struct ScalingHelper {
    /// scale setting in settings
    pub settings_scale: f64,
    /// playfield offset in settings
    pub settings_offset: Vector2,

    /// window size to playfield size scale, scales by settings_scale
    pub scale: f64,

    /// window size from settings
    pub window_size: Vector2,

    /// scaled pos offset for the playfield
    pub scaled_pos_offset: Vector2,

    /// cs size scaled
    pub scaled_cs: f64
}
impl ScalingHelper {
    pub fn new(cs:f32, mode:PlayMode) -> Self {
        let window_size = window_size();

        let (settings_scale, settings_offset) = match mode {
            PlayMode::Standard => Settings::get_mut().standard_settings.get_playfield(),
            _ => {(0.0, Vector2::zero())}
        };
            
        let scale = (window_size.y / FIELD_SIZE.y) * settings_scale;
        let scaled_pos_offset = (window_size - FIELD_SIZE * scale) / 2.0 + settings_offset;


        let cs_base = (1.0 - 0.7 * (cs as f64 - 5.0) / 5.0) / 2.0;
        let scaled_cs = cs_base * scale;

        Self {
            settings_scale,
            settings_offset,
            scale,
            window_size,
            scaled_pos_offset,
            scaled_cs
        }
    }

    /// turn playfield (osu) coords into window coords
    pub fn scale_coords(&self, osu_coords:Vector2) -> Vector2 {
        self.scaled_pos_offset + osu_coords * self.scale
    }
    /// turn window coords into playfield coords
    pub fn descale_coords(&self, window_coords: Vector2) -> Vector2 {
        (window_coords - self.scaled_pos_offset) / self.scale
    }

}