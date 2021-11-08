mod replay;
mod score;
mod user_status;
mod spectator;

pub use score::*;
pub use replay::*;
pub use user_status::*;
pub use spectator::*;

use serde::{Serialize, Deserialize};
use crate::serialization::Serializable;

use PlayMode::*;

#[repr(u8)]
#[derive(Debug,Clone,Copy,PartialEq, Serialize, Deserialize)]
pub enum PlayMode {
    Standard,
    Taiko,
    Catch,
    Mania,
    #[allow(non_camel_case_types)]
    pTyping
}
impl Into<PlayMode> for u8 {
    fn into(self) -> PlayMode {
        match self {
            0 => Standard,
            1 => Taiko,
            2 => Catch,
            3 => Mania,
            4 => pTyping,
            _ => Standard
        }
    }
}
impl Into<u8> for PlayMode {
    fn into(self) -> u8 {
        match self {
            Standard => 0,
            Taiko => 1,
            Catch => 2,
            Mania => 3,
            pTyping => 4,
        }
    }
}
impl Default for PlayMode {
    fn default() -> Self {
        PlayMode::Standard
    }
}

impl Serializable for PlayMode {
    fn read(sr:&mut crate::serialization::SerializationReader) -> Self {
        sr.read_u8().into()
    }
    fn write(&self, sw:&mut crate::serialization::SerializationWriter) {
        let num:u8 = (*self).into();
        sw.write_u8(num)
    }
}