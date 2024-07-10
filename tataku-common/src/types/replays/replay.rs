use crate::prelude::*;
use std::collections::HashMap;

// all versions have the frame data
// v1 had a playmode entry, which was only relevent to taiko
// v2+ has score data included in the replay data
// v4+ has gamemode data (technically in v3 but i messed up whoops)
// v5+ has map time offset
// v6 had breaking changes since we moved the replay to the score (whereas before the score used to be in the replay)
const CURRENT_VERSION:u16 = 6;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Replay {
    /// any extra gameplay variables which are helpful to know
    pub gamemode_data: HashMap<String, String>,

    /// time offset 
    pub offset: f32,

    /// (time, key)
    pub frames: Vec<ReplayFrame>, 
}
impl Replay {
    pub fn new() -> Replay {
        Replay {
            gamemode_data: HashMap::new(),
            offset: 0.0,
            frames: Vec::new(),
        }
    }
}
impl Serializable for Replay {
    fn read(sr: &mut SerializationReader) -> SerializationResult<Self> {
        let mut r = Replay::new();
        
        // all versions wrote the version number
        let version = sr.read_u16()?;

        if version < 6 {
            let _score: Score = sr.read()?;
            let mut r = Replay::new();
                
            r.gamemode_data = sr.read()?;
            r.offset = sr.read()?;
            r.frames = sr.read()?;
            return Ok(r);
        }

        r.gamemode_data = sr.read()?;
        r.offset = sr.read()?;
        r.frames = sr.read()?;
        
        Ok(r)
    }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write(&CURRENT_VERSION); // all versions
        // sw.write(self.playstyle as u8); // removed in v2
        // sw.write(&self.score_data); // added in v2, removed in v6
        sw.write(&self.gamemode_data); // added in v3, fixed in v4
        sw.write(&self.offset); // added in v5

        // println!("writing {} replay frames", self.frames.len());
        sw.write(&self.frames); // all versions
    }
}
