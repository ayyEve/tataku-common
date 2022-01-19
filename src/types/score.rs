use core::fmt;
use serde::{Serialize, Deserialize};
use crate::types::{PlayMode, Replay};
use crate::prelude::*;



const CURRENT_VERSION:u16 = 2;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Score {
    pub version: u16,
    pub username: String,
    pub beatmap_hash: String,
    pub playmode: PlayMode,
    pub score: u64,
    pub combo: u16,
    pub max_combo: u16,
    pub x50: u16,
    pub x100: u16,
    pub x300: u16,
    pub xgeki: u16,
    pub xkatu: u16,
    pub xmiss: u16,
    pub accuracy: f64,
    pub speed: f32,

    /// time diff for actual note hits. if the note wasnt hit, it wont be here
    /// (user_hit_time - correct_time)
    pub hit_timings: Vec<f32>,

    /// this is only used when a replay is stored as a file
    /// and must be set manually
    pub replay_string: Option<String>
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
            x50: 0,
            x100: 0,
            x300: 0,
            xgeki: 0,
            xkatu: 0,
            xmiss: 0,
            accuracy: 0.0,
            speed: 1.0,
            hit_timings: Vec::new(),
            replay_string: None
        }
    }
    pub fn hash(&self) -> String {
        // TODO! lol
        match self.version {
            // v1 hash didnt have the playmode
            1 => format!("{}-{},{},{},{},{},{}", self.beatmap_hash, self.score, self.combo, self.max_combo, self.x100,self.x300,self.xmiss),
            _ => format!("{}-{},{},{},{},{},{},{:?}", self.beatmap_hash, self.score, self.combo, self.max_combo, self.x100,self.x300,self.xmiss,self.playmode)
        }
    }

    ///0-1
    pub fn acc(&self) -> f64 {
        let xmiss = self.xmiss as f64;
        let x100 = self.x100 as f64;
        let x300 = self.x300 as f64;

        let n = (0.5*x100 + x300) / (xmiss + x100 + x300);
        if n.is_nan() {return 0.0}
        n
    }
    pub fn hit_error(&self) -> HitError {
        // from https://gist.github.com/peppy/3a11cb58c856b6af7c1916422f668899
        let mut total = 0.0;
        let mut _total = 0.0;
        let mut total_all = 0.0;
        let mut count = 0.0;
        let mut _count = 0.0;

        for i in self.hit_timings.as_slice() {
            let i = i.clone();
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
        for i in self.hit_timings.as_slice() {
            variance += (i.clone() - mean).powi(2);
        }

        HitError {
            mean,
            early: _total / _count,
            late: total / count,
            deviance: (variance / self.hit_timings.len() as f32).sqrt()
        }
    }

    pub fn hit_miss(&mut self, hit_time:f32, note_time:f32) {
        self.combo = 0;
        self.xmiss += 1;

        self.hit_timings.push(hit_time - note_time);
    }
    pub fn hit50(&mut self, hit_time:f32, note_time:f32) {
        self.combo += 1;
        self.max_combo = self.max_combo.max(self.combo);
        self.x50 += 1;
        self.add_pts(50, true);
        
        self.hit_timings.push(hit_time - note_time);
    }
    pub fn hit100(&mut self, hit_time:f32, note_time:f32) {
        self.combo += 1;
        self.max_combo = self.max_combo.max(self.combo);
        self.x100 += 1;
        self.add_pts(100, true);
        
        self.hit_timings.push(hit_time - note_time);
    }
    pub fn hit_geki(&mut self, hit_time:f32, note_time:f32) {
        self.combo += 1;
        self.max_combo = self.max_combo.max(self.combo);
        self.xgeki += 1;
        self.add_pts(300, true);
        
        self.hit_timings.push(hit_time - note_time);
    }
    pub fn hit300(&mut self, hit_time:f32, note_time:f32) {
        self.combo += 1;
        self.max_combo = self.max_combo.max(self.combo);
        self.x300 += 1;
        self.add_pts(300, true);

        self.hit_timings.push(hit_time - note_time);
    }
    pub fn hit_katu(&mut self, hit_time:f32, note_time:f32) {
        self.combo += 1;
        self.max_combo = self.max_combo.max(self.combo);
        self.xkatu += 1;
        self.add_pts(100, true);

        self.hit_timings.push(hit_time - note_time);
    }

    pub fn add_pts(&mut self, points:u64, affected_by_combo:bool) {
        if affected_by_combo {
            self.score += self.combo as u64 * points;
        } else {
            self.score += points;
        }
    }

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

        Ok(Score {
            version,
            username: sr.read()?,
            beatmap_hash: sr.read()?,
            playmode: sr.read()?,
            score: sr.read()?,
            combo: sr.read()?,
            max_combo: sr.read()?,
            x50: sr.read()?,
            x100: sr.read()?,
            x300: sr.read()?,
            xgeki: sr.read()?,
            xkatu: sr.read()?,
            xmiss: sr.read()?,
            accuracy: sr.read()?,
            speed: version!(2, 0.0),

            hit_timings: Vec::new(),
            replay_string: None
        })
    }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write(CURRENT_VERSION);
        sw.write(self.username.clone());
        sw.write(self.beatmap_hash.clone());
        sw.write(self.playmode);
        sw.write(self.score);
        sw.write(self.combo);
        sw.write(self.max_combo);
        sw.write(self.x50);
        sw.write(self.x100);
        sw.write(self.x300);
        sw.write(self.xgeki);
        sw.write(self.xkatu);
        sw.write(self.xmiss);
        sw.write(self.accuracy);
        sw.write(self.speed);
    }
}
impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ Score (h:{}, m:{:?}, score:{}, combo:{}. max_combo: {}, x50:{},x100:{},x300:{},xgeki:{},xkatu{},xmiss:{},accuracy:{}) }}", 
            self.beatmap_hash, 
            self.playmode,
            self.score,
            self.combo,
            self.max_combo,
            self.x50,
            self.x100,
            self.x300,
            self.xgeki,
            self.xkatu,
            self.xmiss,
            self.accuracy
        )
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