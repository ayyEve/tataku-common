use crate::prelude::*;
use std::collections::HashMap;

const CURRENT_VERSION:u16 = 5;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Replay {
    /// score associated with this replay
    pub score_data: Option<Score>,

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
            score_data: None,
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

        // v1 had a playmode entry, which was only relevent to taiko
        if version == 1 {
            let _playstyle = sr.read_u8()?;
        }
        
        // v2+ has score data included in the replay data
        if version >= 2 {
            r.score_data = sr.read()?;
        }

        // v4+ has gamemode data (technically in v3 but i messed up whoops)
        if version >= 4 {
            r.gamemode_data = sr.read()?;
        }

        // v5+ has map offset
        if version >= 5 {
            r.offset = sr.read()?;
        }

        // all versions have the frame data
        r.frames = sr.read()?;
        
        Ok(r)
    }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write(&CURRENT_VERSION); // all versions
        // sw.write(self.playstyle as u8); // removed in v2
        sw.write(&self.score_data); // added in v2
        sw.write(&self.gamemode_data); // added in v3, fixed in v4
        sw.write(&self.offset); // added in v5

        // println!("writing {} replay frames", self.frames.len());
        sw.write(&self.frames); // all versions
    }
}
