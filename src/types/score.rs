use crate::prelude::*;
use std::collections::HashMap;
use serde::{ Serialize, Deserialize };
use crate::types::{ PlayMode };


const CURRENT_VERSION:u16 = 5;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Score {
    pub version: u16,
    pub username: String,
    pub beatmap_hash: String,
    pub playmode: PlayMode,
    /// time in non-leap seconds since unix_epoch (UTC)
    pub time: u64,

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
}
impl Score {
    pub fn new(beatmap_hash:String, username:String, playmode:PlayMode) -> Score {
        Score {
            version: CURRENT_VERSION,
            username,
            beatmap_hash,
            playmode,
            time: 0,

            score: 0,
            combo: 0,
            max_combo: 0,

            judgments: HashMap::new(),
            accuracy: 0.0,
            speed: 1.0,
            hit_timings: Vec::new(),
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

        let judgments_str = self.judgment_string();
        let Score { beatmap_hash, score, combo, max_combo, playmode, .. } = &self;

        match self.version {
            // v3 still used manual judgments
            4.. => format!("{beatmap_hash}-{score},{combo},{max_combo},{judgments_str},{mods},{playmode}"),
            // v2 didnt have mods
            3 => format!("{beatmap_hash}-{score},{combo},{max_combo},{x100},{x300},{xmiss},{mods},{playmode}"),
            // v1 hash didnt have the playmode
            2 => format!("{beatmap_hash}-{score},{combo},{max_combo},{x100},{x300},{xmiss},{playmode}"),
            1 => format!("{beatmap_hash}-{score},{combo},{max_combo},{x100},{x300},{xmiss}"),
            0 => format!("what"),
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

    pub fn judgment_string(&self) -> String {
        let mut judgments = self.judgments.iter()
            .map(|(key, val)| format!("{key}:{val}"))
            .collect::<Vec<String>>();

        judgments.sort_unstable();
        judgments.join("|")
    }

    pub fn judgments_from_string(judgment_string: &String) -> HashMap<String, u16> {
        let mut judgments = HashMap::new();
        
        let entries = judgment_string.split("|");
        for entry in entries {
            let mut split = entry.split(":");
            if let Some((key, val)) = split.next().zip(split.next()) {
                let key = key.to_owned();
                let val = val.parse().unwrap();
                judgments.insert(key, val);
            }
        }

        judgments
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

        let username = sr.read()?;
        let beatmap_hash = sr.read()?;
        let playmode = sr.read()?;
        let time = version!(5, 0); // v5 added time

        let score = sr.read()?;
        let combo = sr.read()?;
        let max_combo = sr.read()?;

        // before version 4, judgments were stored manually
        let mut judgments = HashMap::new();
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
            time,
            
            score,
            combo,
            max_combo,
            judgments,
            accuracy: sr.read()?,
            speed: version!(2, 0.0),

            hit_timings: Vec::new(),
            mods_string: version!(3, None)
        })
    }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write(CURRENT_VERSION);
        sw.write(self.username.clone());
        sw.write(self.beatmap_hash.clone());
        sw.write(self.playmode.clone());
        sw.write(self.time);

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