use serde::Deserialize;



#[derive(Deserialize)]
#[serde(rename_all="PascalCase")]
pub struct QuaverBeatmap {
    pub audio_file: String,
    pub song_preview_time: f32,
    pub background_file: String,

    // dunno if they can be negative
    pub map_id: i32,
    pub set_id: i32,

    pub mode: QuaverKeys,

    pub title: String,
    pub artist: String,
    pub source: String,
    pub tags: String,
    pub creator: String,
    pub difficulty_name: String,
    pub description: String,

    // pub editor_layers: Vec<?>,
    // pub audio_samples: Vec<?>,
    // pub sound_effects: Vec<?>,
    
    pub timing_points: Vec<QuaverTimingPoint>,
    // pub slider_velocities: Vec<?>,
    pub hit_objects: Vec<QuaverNote>
}



#[derive(Deserialize)]
pub enum QuaverKeys {
    Keys5,
    Keys7,
}


#[derive(Deserialize)]
#[serde(rename_all="PascalCase")]
pub struct QuaverTimingPoint {
    pub start_time: f32,
    pub bpm: f32
}

#[derive(Deserialize)]
#[serde(rename_all="PascalCase")]
pub struct QuaverNote {
    start_time: f32,
    lane: u8,
    // key_sounds: Vec<?>
}