use taiko_rs_common::types::PlayMode;

use crate::game::managers::ModManager;


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

    pub hp: f32,
    pub od: f32,
    pub cs: f32,
    pub ar: f32,
    // pub sr: f64,

    pub slider_multiplier: f32,
    pub slider_tick_rate: f32,

    pub bpm_min: f32,
    pub bpm_max: f32,
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
            bpm_min: 0.0,
            bpm_max: 0.0
        }
    }
    pub fn do_checks(&mut self) {
        if self.ar < 0.0 {self.ar = self.od}
    }

    fn mins(&self, speed:f32) -> f32 {
        ((self.duration / speed) / 60000.0).floor() 
    }
    fn secs(&self, speed:f32) -> f32 {
        let mins = self.mins(speed);
        let remaining_ms = (self.duration / speed) - mins * 60000.0;
        (remaining_ms / 1000.0).floor()
    }

    /// get the title string with the version
    pub fn version_string(&self) -> String {
        format!("{} - {} [{}]", self.artist, self.title, self.version)  
    }

    /// get the difficulty string (od, hp, sr, bpm, len)
    pub fn diff_string(&self) -> String {
        let mods = ModManager::get();
        let symb = if mods.speed > 1.0 {"+"} else if mods.speed < 1.0 {"-"} else {""};

        // format!("od: {:.2} hp: {:.2}, {:.2}*, {}:{}", self.od, self.hp, self.sr, self.mins, self.secs)
        let mut secs = format!("{}", self.secs(mods.speed));
        if secs.len() == 1 {secs = format!("0{}",secs)}

        let mut txt = format!(
            "od: {:.2}{} hp: {:.2}{}, dur: {}:{}", 
            self.od, symb,
            self.hp, symb,
            self.mins(mods.speed), secs
        );

        // make sure at least one has a value
        if self.bpm_min != 0.0 || self.bpm_max != 0.0 {
            // one bpm
            if self.bpm_min == self.bpm_max {
                txt += &format!(" bpm: {:.2}", self.bpm_min * mods.speed);
            } else { // multi bpm
                // i think i had it backwards when setting, just make sure its the right way :/
                let min = self.bpm_min.min(self.bpm_max);
                let max = self.bpm_max.max(self.bpm_min);
                txt += &format!(" bpm: {:.2}-{:.2}", min * mods.speed, max * mods.speed);
            }
        }

        txt
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

