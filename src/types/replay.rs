use crate::serialization::{Serializable, SerializationReader, SerializationWriter};

const CURRENT_VERSION:u16 = 1;

#[derive(Clone, Debug, Default)]
pub struct Replay {
    /// (time, key)
    pub frames: Vec<(f32, ReplayFrame)>, 
    pub playstyle: Playstyle,

    /// used internally, not stored externally
    pub speed: f32
}
impl Replay {
    pub fn new() -> Replay {
        Replay {
            frames: Vec::new(),
            playstyle: Playstyle::None,
            speed: 1.0
        }
    }
}
impl Serializable for Replay {
    fn read(sr: &mut SerializationReader) -> Self {
        let mut r = Replay::new();
        
        let _version = sr.read_u16();
        r.playstyle = sr.read_u8().into();
        r.frames = sr.read();
        
        r
    }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write(CURRENT_VERSION);
        sw.write(self.playstyle as u8);
        println!("writing {} replay frames", self.frames.len());
        sw.write(&self.frames);
    }
}



#[derive(Clone, Debug, Copy)]
pub enum Playstyle {
    None = 0,
    KDDK = 1,
    KKDD = 2,
    DDKK = 3
}
impl Into<u8> for Playstyle {
    fn into(self) -> u8 {self as u8}
}
impl From<u8> for Playstyle {
    fn from(n: u8) -> Self {
        use Playstyle::*;
        match n {
            0 => KDDK,
            1 => KKDD,
            2 => DDKK,
            _ => KKDD
        }
    }
}
impl Default for Playstyle {
    fn default() -> Self {
        Self::KDDK
    }
}


#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
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

    /// doubles as std left
    Left = 30,
    /// doubles as std right
    Right = 31,
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
    fn read(sr:&mut SerializationReader) -> Self {sr.read_u8().into()}
    fn write(&self, sw:&mut SerializationWriter) {sw.write_u8(self.clone() as u8)}
}

#[derive(Clone, Copy, Debug)]
pub enum ReplayFrame {
    Press(KeyPress),
    Release(KeyPress),
    MousePos(f32, f32)
}
impl Serializable for ReplayFrame {
    fn read(sr:&mut SerializationReader) -> Self {
        use ReplayFrame::*;
        match sr.read_u8() {
            0 => Press(sr.read()),
            1 => Release(sr.read()),
            2 => MousePos(sr.read(), sr.read()),
            _ => panic!("error reading replay frame type")
        }
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