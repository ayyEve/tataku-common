use core::fmt;

use crate::types::replay::Replay;


use crate::serialization::{Serializable, SerializationReader, SerializationWriter};

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
