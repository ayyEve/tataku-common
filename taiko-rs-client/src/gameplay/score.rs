use std::fmt;

use taiko_rs_common::serialization::{Serializable,SerializationReader,SerializationWriter};
use crate::game::Settings;
use crate::gameplay::Replay;

const CURRENT_VERSION: u16 = 1;

#[derive(Clone, Debug)]
pub struct Score {
    pub username: String,
    pub beatmap_hash: String,
    pub score: u64,
    pub combo: u16,
    pub max_combo: u16,
    pub x100: u16,
    pub x300: u16,
    pub xmiss: u16,

    pub replay: Replay,

    /// time diff for actual note hits. if the note wasnt hit, it wont be here
    /// (user_hit_time - correct_time)
    pub hit_timings: Vec<i32>
}
impl Score {
    pub fn new(hash:String) -> Score {
        let username = Settings::get().username.clone();
        Score {
            username,
            beatmap_hash: hash,
            score: 0,
            combo: 0,
            max_combo: 0,
            x100: 0,
            x300: 0,
            xmiss: 0,

            replay: Replay::new(),
            hit_timings: Vec::new()
        }
    }


    ///0-1
    pub fn acc(&self) -> f64 {
        let xmiss = self.xmiss as f64;
        let x100 = self.x100 as f64;
        let x300 = self.x300 as f64;

        let n = (0.5*x100 + x300) / (xmiss + x100 + x300);
        if n.is_nan() {
            return 0.0;
        }
        n
    }
    pub fn hit_error(&self) -> HitError {
        // from https://gist.github.com/peppy/3a11cb58c856b6af7c1916422f668899
        let mut total = 0;
        let mut _total = 0;
        let mut total_all = 0;
        let mut count = 0;
        let mut _count = 0;

        for i in self.hit_timings.as_slice() {
            let i = i.clone();
            total_all += i;

            if i > 0 {
                total += i;
                count += 1;
            } else {
                _total += i;
                _count += 1;
            }
        }

        let average = total_all as f64 / self.hit_timings.len() as f64;
        let mut variance = 0.0;
        for i in self.hit_timings.as_slice() {
            variance += (i.clone() as f64 - average).powi(2);
        }

        HitError {
            early: _total as f64 / _count as f64,
            late: total as f64 / count as f64,
            deviance: (variance / self.hit_timings.len() as f64).sqrt()
        }
    }

    pub fn hit_miss(&mut self, hit_time:u64, note_time:u64) {
        self.combo = 0;
        self.xmiss += 1;

        self.hit_timings.push(hit_time as i32 - note_time as i32);
    }
    pub fn hit100(&mut self, hit_time:u64, note_time:u64) {
        self.combo += 1;
        self.max_combo = self.max_combo.max(self.combo);
        self.x100 += 1;
        self.add_pts(100, true);
        
        self.hit_timings.push(hit_time as i32 - note_time as i32);
    }
    pub fn hit300(&mut self, hit_time:u64, note_time:u64) {
        self.combo += 1;
        self.max_combo = self.max_combo.max(self.combo);
        self.x300 += 1;
        self.add_pts(300, true);

        self.hit_timings.push(hit_time as i32 - note_time as i32);
    }

    pub fn add_pts(&mut self, points:u64, affected_by_combo:bool) {
        if affected_by_combo {
            self.score += (self.combo as f64 * points as f64) as u64;
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
            score: sr.read(),
            combo: sr.read(),
            max_combo: sr.read(),
            x100: sr.read(),
            x300: sr.read(),
            xmiss: sr.read(),
            replay: sr.read(),
            hit_timings: Vec::new()
        }
    }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write(CURRENT_VERSION);
        sw.write(self.username.clone());
        sw.write(self.beatmap_hash.clone());
        sw.write(self.score);
        sw.write(self.combo);
        sw.write(self.max_combo);
        sw.write(self.x100);
        sw.write(self.x300);
        sw.write(self.xmiss);
        sw.write(self.replay.clone());
    }
}
impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ Score (h:{}, score:{}, combo:{}. max_combo: {}, x100:{},x300:{},xmiss:{}) }}", 
            self.beatmap_hash, 
            self.score,
            self.combo,
            self.max_combo,
            self.x100,
            self.x300,
            self.xmiss
        )
    }
}


#[derive(Debug)]
pub enum ScoreHit {
    None,
    Miss,
    X100,
    X300,
    /// score increment, consume the object
    Other(u32, bool)
}


/// helper struct
#[derive(Copy, Clone, Debug)]
pub struct HitError {
    pub early: f64,
    pub late: f64,
    pub deviance: f64
}