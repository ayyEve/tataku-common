use core::fmt;
use crate::serialization::{Serializable, SerializationReader, SerializationWriter};

use super::PlayMode;

const CURRENT_VERSION: u16 = 1;

#[derive(Clone, Debug)]
pub struct Score {
    pub username: String,
    pub beatmap_hash: String,
    pub playmode: PlayMode,
    pub score: u64,
    pub combo: u16,
    pub max_combo: u16,
    pub x50: u16,
    pub x100: u16,
    pub x300: u16,
    pub geki: u16,
    pub katu: u16,
    pub xmiss: u16,

    /// time diff for actual note hits. if the note wasnt hit, it wont be here
    /// (user_hit_time - correct_time)
    pub hit_timings: Vec<f32>,
}
impl Score {
    pub fn new(beatmap_hash:String, username:String, playmode:PlayMode) -> Score {
        Score {
            username,
            beatmap_hash,
            playmode,
            score: 0,
            combo: 0,
            max_combo: 0,
            x50: 0,
            x100: 0,
            x300: 0,
            geki: 0,
            katu: 0,
            xmiss: 0,
            hit_timings: Vec::new()
        }
    }
    pub fn hash(&self) -> String {
        // TODO! lol
        format!("{}-{},{},{},{},{},{}", self.beatmap_hash, self.score, self.combo, self.max_combo, self.x100,self.x300,self.xmiss)
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
        self.x100 += 1;
        self.add_pts(100, true);
        
        self.hit_timings.push(hit_time - note_time);
    }
    pub fn hit100(&mut self, hit_time:f32, note_time:f32) {
        self.combo += 1;
        self.max_combo = self.max_combo.max(self.combo);
        self.x100 += 1;
        self.add_pts(100, true);
        
        self.hit_timings.push(hit_time - note_time);
    }
    pub fn hit300(&mut self, hit_time:f32, note_time:f32) {
        self.combo += 1;
        self.max_combo = self.max_combo.max(self.combo);
        self.x300 += 1;
        self.add_pts(300, true);

        self.hit_timings.push(hit_time - note_time);
    }

    pub fn add_pts(&mut self, points:u64, affected_by_combo:bool) {
        if affected_by_combo {
            self.score += self.combo as u64 * points;
        } else {
            self.score += points;
        }
    }
}
impl Serializable for Score {
    fn read(sr: &mut SerializationReader) -> Self {
        let _version = sr.read_u16();

        Score {
            username: sr.read(),
            beatmap_hash: sr.read(),
            playmode: sr.read(),
            score: sr.read(),
            combo: sr.read(),
            max_combo: sr.read(),
            x50: sr.read(),
            x100: sr.read(),
            x300: sr.read(),
            geki: sr.read(),
            katu: sr.read(),
            xmiss: sr.read(),
            hit_timings: Vec::new()
        }
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
        sw.write(self.geki);
        sw.write(self.katu);
        sw.write(self.xmiss);
    }
}
impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ Score (h:{}, m:{:?}, score:{}, combo:{}. max_combo: {}, x50:{},x100:{},x300:{},geki:{},katu:{},xmiss:{}) }}", 
            self.beatmap_hash, 
            self.playmode,
            self.score,
            self.combo,
            self.max_combo,
            self.x50,
            self.x100,
            self.x300,
            self.geki,
            self.katu,
            self.xmiss
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