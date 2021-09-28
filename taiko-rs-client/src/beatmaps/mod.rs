use std::path::Path;

use crate::errors::{BeatmapError, TaikoError, TaikoResult};

use self::{common::{BeatmapMeta, TaikoRsBeatmap, TimingPoint}, osu::OsuBeatmap};

pub mod osu;
pub mod quaver;
pub mod common;
pub mod adofai;




pub enum Beatmap {
    /// used for defaults
    None,
    /// osu file
    Osu(osu::OsuBeatmap),
    /// quaver file
    Quaver(quaver::QuaverBeatmap),
    /// adofai file
    Adofai(adofai::AdofaiBeatmap)
}
impl Beatmap {
    pub fn load<F:AsRef<Path>>(path: F) -> TaikoResult<Beatmap> {
        let path = path.as_ref();
        if path.extension().is_none() {return Err(TaikoError::Beatmap(BeatmapError::InvalidFile))}
        
        
        match path.extension().unwrap().to_str().unwrap() {
            "osu" => Ok(Beatmap::Osu(OsuBeatmap::load(path.to_str().unwrap().to_owned()))),
            
            _ => Err(TaikoError::Beatmap(BeatmapError::InvalidFile)),
        }
    }

    pub fn from_metadata(meta: &BeatmapMeta) -> TaikoResult<Beatmap> {
        Self::load(&meta.file_path)
    }
}
impl Default for Beatmap {
    fn default() -> Self {Beatmap::None}
}



impl TaikoRsBeatmap for Beatmap {
    fn hash(&self) -> String {
        match self {
            Beatmap::None => todo!(),
            Beatmap::Osu(map) => map.hash(),
            Beatmap::Quaver(_) => todo!(),
            Beatmap::Adofai(_) => todo!(),
        }
    }

    fn get_timing_points(&self) -> Vec<TimingPoint> {
        match self {
            Beatmap::None => todo!(),
            Beatmap::Osu(map) => map.get_timing_points(),
            Beatmap::Quaver(_) => todo!(),
            Beatmap::Adofai(_) => todo!(),
        }
    }

    fn get_beatmap_meta(&self) -> BeatmapMeta {
        match self {
            Beatmap::None => todo!(),
            Beatmap::Osu(map) => map.get_beatmap_meta(),
            Beatmap::Quaver(_) => todo!(),
            Beatmap::Adofai(_) => todo!(),
        }
    }

    fn playmode(&self, incoming: taiko_rs_common::types::PlayMode) -> taiko_rs_common::types::PlayMode {
        match self {
            Beatmap::None => todo!(),
            Beatmap::Osu(map) => map.playmode(incoming),
            Beatmap::Quaver(_) => todo!(),
            Beatmap::Adofai(_) => todo!(),
        }
    }

    fn slider_velocity_at(&self, time:f32) -> f32 {
        match self {
            Beatmap::None => todo!(),
            Beatmap::Osu(map) => map.slider_velocity_at(time),
            Beatmap::Quaver(_) => todo!(),
            Beatmap::Adofai(_) => todo!(),
        }
    }

    fn beat_length_at(&self, time:f32, allow_multiplier:bool) -> f32 {
        match self {
            Beatmap::None => todo!(),
            Beatmap::Osu(map) => map.beat_length_at(time, allow_multiplier),
            Beatmap::Quaver(_) => todo!(),
            Beatmap::Adofai(_) => todo!(),
        }
    }

    fn control_point_at(&self, time:f32) -> TimingPoint {
        match self {
            Beatmap::None => todo!(),
            Beatmap::Osu(map) => map.control_point_at(time),
            Beatmap::Quaver(_) => todo!(),
            Beatmap::Adofai(_) => todo!(),
        }
    }
}