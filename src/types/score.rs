use crate::prelude::*;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::types::{PlayMode, Replay};


const CURRENT_VERSION:u16 = 4;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Score {
    pub version: u16,
    pub username: String,
    pub beatmap_hash: String,
    pub playmode: PlayMode,
    pub score: u64,
    pub combo: u16,
    pub max_combo: u16,

    pub judgments: HashMap<String, u16>,
    // pub x50: u16,
    // pub x100: u16,
    // pub x300: u16,
    // pub xgeki: u16,
    // pub xkatu: u16,
    // pub xmiss: u16,

    pub accuracy: f64,
    pub speed: f32,
    pub mods_string: Option<String>,

    /// time diff for actual note hits. if the note wasnt hit, it wont be here
    /// (user_hit_time - correct_time)
    pub hit_timings: Vec<f32>,

    /// this is only used when a replay is stored as a file
    /// and must be set manually
    pub replay_string: Option<String>,
}
impl Score {
    pub fn new(beatmap_hash:String, username:String, playmode:PlayMode) -> Score {
        Score {
            version: CURRENT_VERSION,
            username,
            beatmap_hash,
            playmode,
            score: 0,
            combo: 0,
            max_combo: 0,

            judgments: HashMap::new(),
            accuracy: 0.0,
            speed: 1.0,
            hit_timings: Vec::new(),
            replay_string: None,
            mods_string: None,
        }
    }
    pub fn hash(&self) -> String {
        let mods = if let Some(m) = &self.mods_string {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(&m, &mut hasher);
            format!("{:x}", std::hash::Hasher::finish(&hasher))
        } else {
            format!("None")
        };

        // TODO: lol
        let x100 = self.judgments.get("x100")  .map(|n|*n).unwrap_or_default();
        let x300 = self.judgments.get("x300")  .map(|n|*n).unwrap_or_default();
        let xmiss = self.judgments.get("xmiss").map(|n|*n).unwrap_or_default();

        let judgments_str = {
            let mut s = String::new();
            for (k, v) in self.judgments.iter() {
                s += &format!("{k}:{v}");
            }
            s
        };

        let Score { beatmap_hash, score, combo, max_combo, playmode, .. } = &self;

        match self.version {
            // v3 still used manual judgments
            4 => format!("{beatmap_hash}-{score},{combo},{max_combo},{judgments_str},{mods},{playmode}"),
            // v2 didnt have mods
            3 => format!("{beatmap_hash}-{score},{combo},{max_combo},{x100},{x300},{xmiss},{mods},{playmode}"),
            // v1 hash didnt have the playmode
            2 => format!("{beatmap_hash}-{score},{combo},{max_combo},{x100},{x300},{xmiss},{playmode}"),
            1 => format!("{beatmap_hash}-{score},{combo},{max_combo},{x100},{x300},{xmiss}"),
            _ => format!("unknown?!?!"),
        }
    }

    pub fn hit_error(&self) -> HitError {
        // from https://gist.github.com/peppy/3a11cb58c856b6af7c1916422f668899
        let mut total = 0.0;
        let mut _total = 0.0;
        let mut total_all = 0.0;
        let mut count = 0.0;
        let mut _count = 0.0;

        for &i in self.hit_timings.iter() {
            total_all += i;

            if i > 0.0 {
                total += i;
                count += 1.0;
            } else {
                _total += i;
                _count += 1.0;
            }
        }

        let mean = total_all / self.hit_timings.len() as f32;
        let mut variance = 0.0;
        for &i in self.hit_timings.iter() {
            variance += (i - mean).powi(2);
        }

        HitError {
            mean,
            early: _total / _count,
            late: total / count,
            deviance: (variance / self.hit_timings.len() as f32).sqrt()
        }
    }

    // pub fn hit_miss(&mut self, hit_time:f32, note_time:f32) {
    //     self.combo = 0;
    //     self.xmiss += 1;

    //     self.hit_timings.push(hit_time - note_time);
    // }
    // pub fn hit50(&mut self, hit_time:f32, note_time:f32) {
    //     self.combo += 1;
    //     self.max_combo = self.max_combo.max(self.combo);
    //     self.x50 += 1;
    //     self.add_pts(50, true);
        
    //     self.hit_timings.push(hit_time - note_time);
    // }
    // pub fn hit100(&mut self, hit_time:f32, note_time:f32) {
    //     self.combo += 1;
    //     self.max_combo = self.max_combo.max(self.combo);
    //     self.x100 += 1;
    //     self.add_pts(100, true);
        
    //     self.hit_timings.push(hit_time - note_time);
    // }
    // pub fn hit_geki(&mut self, hit_time:f32, note_time:f32) {
    //     self.combo += 1;
    //     self.max_combo = self.max_combo.max(self.combo);
    //     self.xgeki += 1;
    //     self.add_pts(300, true);
        
    //     self.hit_timings.push(hit_time - note_time);
    // }
    // pub fn hit300(&mut self, hit_time:f32, note_time:f32) {
    //     self.combo += 1;
    //     self.max_combo = self.max_combo.max(self.combo);
    //     self.x300 += 1;
    //     self.add_pts(300, true);

    //     self.hit_timings.push(hit_time - note_time);
    // }
    // pub fn hit_katu(&mut self, hit_time:f32, note_time:f32) {
    //     self.combo += 1;
    //     self.max_combo = self.max_combo.max(self.combo);
    //     self.xkatu += 1;
    //     self.add_pts(100, true);

    //     self.hit_timings.push(hit_time - note_time);
    // }

    // pub fn add_pts(&mut self, points:u64, affected_by_combo:bool) {
    //     if affected_by_combo {
    //         self.score += self.combo as u64 * points;
    //     } else {
    //         self.score += points;
    //     }
    // }

    /// insert a replay into this score object
    /// should really only be used when saving a replay, as it will probably increase the ram usage quite a bit
    pub fn insert_replay(&mut self, _replay: Replay) {
        todo!()
        // let mut writer = SerializationWriter::new();
        // writer.write(replay);
        // let replay_bytes = writer.data();
        // let encoded = base64::encode(replay_bytes);
        // self.replay_string = Some(encoded);
    }
}
impl Serializable for Score {
    fn read(sr: &mut SerializationReader) -> SerializationResult<Self> {
        let version = sr.read_u16()?;
        macro_rules! version {
            ($v:expr, $def:expr) => {
                if version >= $v {
                    sr.read()?
                } else {
                    $def
                }
            };
        }

        let mut judgments = HashMap::new();

        let username = sr.read()?;
        let beatmap_hash = sr.read()?;
        let playmode = sr.read()?;

        let score = sr.read()?;
        let combo = sr.read()?;
        let max_combo = sr.read()?;

        // before version 4, judgments were stored manually
        if version < 4 {
            for i in [
                "x50",
                "x100",
                "x300",
                "xgeki",
                "xkatu",
                "xmiss"
            ] {
                let val:u16 = sr.read()?;
                judgments.insert(i.to_owned(), val);
            }
        } else {
            let count:usize = sr.read()?;
            for _ in 0..count {
                let key = sr.read()?;
                let val = sr.read()?;
                judgments.insert(key, val);
            }
        }


        Ok(Score {
            version,
            username,
            beatmap_hash,
            playmode,
            score,
            combo,
            max_combo,
            judgments,
            accuracy: sr.read()?,
            speed: version!(2, 0.0),

            hit_timings: Vec::new(),
            replay_string: None,
            mods_string: version!(3, None)
        })
    }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write(CURRENT_VERSION);
        sw.write(self.username.clone());
        sw.write(self.beatmap_hash.clone());
        sw.write(self.playmode.clone());
        sw.write(self.score);
        sw.write(self.combo);
        sw.write(self.max_combo);
        sw.write(self.judgments.clone());
        sw.write(self.accuracy);
        sw.write(self.speed);
        sw.write(self.mods_string.clone());
    }
}



#[derive(Copy, Clone, Debug)]
pub enum ScoreHit {
    None,
    Miss,
    X50,
    X100,
    X300,
    Xgeki,
    Xkatu,
    /// score increment, consume the object
    Other(u32, bool)
}


/// helper struct
#[derive(Copy, Clone, Debug)]
pub struct HitError {
    pub mean: f32,
    pub early: f32,
    pub late: f32,
    pub deviance: f32
}