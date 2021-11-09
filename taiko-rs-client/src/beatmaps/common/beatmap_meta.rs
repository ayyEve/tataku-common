use taiko_rs_common::types::PlayMode;


// contains beatmap info unrelated to notes and timing points, etc
#[derive(Clone, Debug, Default)]
pub struct BeatmapMeta {
    pub file_path: String,
    pub beatmap_hash: String,

    pub beatmap_version: u8,
    pub mode: PlayMode,
    pub artist: String,
    pub title: String,
    pub artist_unicode: String,
    pub title_unicode: String,
    pub creator: String,
    pub version: String,
    pub audio_filename: String,
    pub image_filename: String,
    pub audio_preview: f32,

    pub duration: f32, // time in ms from first note to last note
    /// song duration mins, used for display
    pub mins: u8,
    /// song duration seconds, used for display
    pub secs: u8,

    pub hp: f32,
    pub od: f32,
    pub cs: f32,
    pub ar: f32,
    // pub sr: f64,

    pub slider_multiplier: f32,
    pub slider_tick_rate: f32
}
impl BeatmapMeta {
    pub fn new(file_path:String, beatmap_hash:String) -> BeatmapMeta {
        let unknown = "Unknown".to_owned();

        BeatmapMeta {
            file_path,
            beatmap_hash,
            beatmap_version: 0,
            mode: PlayMode::Standard,
            artist: unknown.clone(),
            title: unknown.clone(),
            artist_unicode: unknown.clone(),
            title_unicode: unknown.clone(),
            creator: unknown.clone(),
            version: unknown.clone(),
            audio_filename: String::new(),
            image_filename: String::new(),
            audio_preview: 0.0,
            hp: -1.0,
            od: -1.0,
            ar: -1.0,
            cs: -1.0,
            slider_multiplier: 1.4,
            slider_tick_rate: 1.0,

            duration: 0.0,
            mins: 0,
            secs: 0,
        }
    }
    pub fn do_checks(&mut self) {
        if self.ar < 0.0 {self.ar = self.od}
    }

    pub fn set_dur(&mut self, duration: f32) {
        self.duration = duration;
        self.mins = (self.duration / 60000.0).floor() as u8;
        self.secs = ((self.duration / 1000.0) % (self.mins as f32 * 60.0)).floor() as u8;
    }

    /// get the title string with the version
    pub fn version_string(&self) -> String {
        format!("{} - {} [{}]", self.artist, self.title, self.version)  
    }

    /// get the difficulty string (od, hp, sr)
    pub fn diff_string(&self) -> String {
        // format!("od: {:.2} hp: {:.2}, {:.2}*, {}:{}", self.od, self.hp, self.sr, self.mins, self.secs)
        format!("od: {:.2} hp: {:.2}, {}:{}", self.od, self.hp, self.mins, self.secs)
    }

    pub fn filter(&self, filter_str: &str) -> bool {
        self.artist.to_ascii_lowercase().contains(filter_str) 
        || self.artist_unicode.to_ascii_lowercase().contains(filter_str) 
        || self.title.to_ascii_lowercase().contains(filter_str) 
        || self.title_unicode.to_ascii_lowercase().contains(filter_str) 
        || self.creator.to_ascii_lowercase().contains(filter_str) 
        || self.version.to_ascii_lowercase().contains(filter_str) 
    }

    pub fn check_mode_override(&self, override_mode:PlayMode) -> PlayMode {
        if self.mode == PlayMode::Standard {
            override_mode
        } else {
            self.mode
        }
    }
}


// might use this later idk
// pub trait IntoSets {
//     fn sort_into_sets(&self) -> Vec<Vec<BeatmapMeta>>;
// }
// impl IntoSets for Vec<BeatmapMeta> {
//     fn sort_into_sets(&self) -> Vec<Vec<BeatmapMeta>> {
//         todo!()
//     }
// }

