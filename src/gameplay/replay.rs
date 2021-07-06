use taiko_rs_common::serialization::{Serializable,SerializationReader,SerializationWriter};

const CURRENT_VERSION:u16 = 1;

#[derive(Clone, Debug)]
pub struct Replay {
    /// (time, key)
    pub presses: Vec<(i64, KeyPress)>, 
    pub playstyle: Playstyle
}
impl Replay {
    pub fn new() -> Replay {
        Replay {
            presses: Vec::new(),
            playstyle: Playstyle::KDDK,
        }
    }
}
impl Serializable for Replay {
    fn read(sr: &mut SerializationReader) -> Self {
        let mut r = Replay::new();
        
        let _version = sr.read_u16();
        r.playstyle = sr.read_u8().into();
        let mut count:u64 = sr.read_u64();
        println!("reading {} replay frames", count);

        while count > 0 {
            count -= 1;

            let time:i64 = sr.read_i64();
            let key:KeyPress = sr.read_u8().into();
            r.presses.push((time, key));
        }
        
        r
    }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write(CURRENT_VERSION);
        sw.write(self.playstyle as u8);
        sw.write(self.presses.len() as u64);
        println!("writing {} replay frames", self.presses.len());

        for (time, key) in self.presses.as_slice() {
            sw.write(time.to_owned());
            sw.write(key.to_owned() as u8);
        }
    }
}


#[derive(Clone, Debug, Copy)]
pub enum Playstyle {
    KDDK = 0,
    KKDD = 1,
    DDKK = 2
}
impl Into<u8> for Playstyle {
    fn into(self) -> u8 {
        self as u8
    }
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

#[derive(Clone, Debug, Copy)]
pub enum KeyPress {
    LeftKat = 0,
    LeftDon = 1,
    RightDon = 2,
    RightKat = 3
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
            0 => LeftKat,
            1 => LeftDon,
            2 => RightDon,
            3 => RightKat,

            _ => LeftKat // maybe it should panic instead?
        }
    }
}

impl Into<super::HitType> for KeyPress {
    fn into(self) -> super::HitType {
        match self {
            KeyPress::LeftKat|KeyPress::RightKat => super::HitType::Kat,
            KeyPress::LeftDon|KeyPress::RightDon => super::HitType::Don,
        }
    }
}