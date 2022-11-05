use crate::prelude::*;
use std::collections::HashMap;

const CURRENT_VERSION:u16 = 4;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Replay {
    /// score associated with this replay
    pub score_data: Option<Score>,

    /// any extra gameplay variables which are helpful to know
    pub gamemode_data: HashMap<String, String>,

    /// (time, key)
    pub frames: Vec<(f32, ReplayFrame)>, 
}
impl Replay {
    pub fn new() -> Replay {
        Replay {
            frames: Vec::new(),
            // playstyle: Playstyle::None,
            score_data: None,
            // speed: 1.0
            gamemode_data: HashMap::new()
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

        // all versions have the frame data
        r.frames = sr.read()?;
        
        Ok(r)
    }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write(CURRENT_VERSION); // all versions
        // sw.write(self.playstyle as u8); // removed in v2
        sw.write(self.score_data.clone()); // added in v2
        sw.write(self.gamemode_data.clone()); // added in v3, fixed in v4

        // println!("writing {} replay frames", self.frames.len());
        sw.write(&self.frames); // all versions
    }
}


#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyPress {
    LeftKat = 0,
    LeftDon = 1,
    RightDon = 2,
    RightKat = 3,

    Mania1 = 4,
    Mania2 = 5,
    Mania3 = 6,
    Mania4 = 7,
    Mania5 = 8,
    Mania6 = 9,
    Mania7 = 10,
    Mania8 = 11,
    Mania9 = 12,

    /// doubles as osu left
    Left = 30,
    /// doubles as osu right
    Right = 31,
    /// doubles as osu smoke
    Dash = 32,

    LeftMouse = 33,
    RightMouse = 34,



    Unknown = 255
}
impl Into<u8> for KeyPress {
    fn into(self) -> u8 {
        self as u8
    }
}
impl From<u8> for KeyPress {
    fn from(n: u8) -> Self {
        use KeyPress::*;
        match n {
            // taiko
            0 => LeftKat,
            1 => LeftDon,
            2 => RightDon,
            3 => RightKat,

            // mania
            4 => Mania1,
            5 => Mania2,
            6 => Mania3,
            7 => Mania4,
            8 => Mania5,
            9 => Mania6,
            10 => Mania7,
            11 => Mania8,
            12 => Mania9,

            30 => Left,
            31 => Right,
            32 => Dash,



            255 => Unknown,
            _ => Unknown
        }
    }
}

impl Serializable for KeyPress {
    fn read(sr:&mut SerializationReader) -> SerializationResult<Self> {Ok(sr.read_u8()?.into())}
    fn write(&self, sw:&mut SerializationWriter) {sw.write_u8(self.clone() as u8)}
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ReplayFrame {
    Press(KeyPress),
    Release(KeyPress),
    MousePos(f32, f32)
}
impl Serializable for ReplayFrame {
    fn read(sr:&mut SerializationReader) -> SerializationResult<Self> {
        use ReplayFrame::*;
        Ok(match sr.read_u8()? {
            0 => Press(sr.read()?),
            1 => Release(sr.read()?),
            2 => MousePos(sr.read()?, sr.read()?),
            _ => panic!("error reading replay frame type")
        })
    }

    fn write(&self, sw:&mut SerializationWriter) {
        use ReplayFrame::*;
        match self {
            Press(k) => {
                sw.write_u8(0);
                sw.write(*k);
            }
            Release(k) => {
                sw.write_u8(1);
                sw.write(*k);
            }
            MousePos(x,y) => {
                sw.write_u8(2);
                sw.write(*x);
                sw.write(*y);
            }
        }
    }
}
