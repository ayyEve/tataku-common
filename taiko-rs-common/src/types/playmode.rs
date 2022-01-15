use crate::prelude::*;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PlayMode {
    Standard,
    Taiko,
    Catch,
    Mania,
    Adofai,
    #[allow(non_camel_case_types)]
    pTyping
}
impl Into<PlayMode> for u8 {
    fn into(self) -> PlayMode {
        match self {
            0 => PlayMode::Standard,
            1 => PlayMode::Taiko,
            2 => PlayMode::Catch,
            3 => PlayMode::Mania,
            4 => PlayMode::pTyping,
            5 => PlayMode::Adofai,
            _ => PlayMode::Standard
        }
    }
}
impl Into<u8> for PlayMode {
    fn into(self) -> u8 {
        match self {
            PlayMode::Standard => 0,
            PlayMode::Taiko => 1,
            PlayMode::Catch => 2,
            PlayMode::Mania => 3,
            PlayMode::pTyping => 4,
            PlayMode::Adofai => 5
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
