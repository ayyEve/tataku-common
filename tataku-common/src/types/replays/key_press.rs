use crate::prelude::*;

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
    Mania10 = 13,

    /// doubles as osu left
    Left = 30,
    /// doubles as osu right
    Right = 31,
    /// doubles as osu smoke
    Dash = 32,

    LeftMouse = 33,
    RightMouse = 34,


    SkipIntro = 254,
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


            254 => SkipIntro,
            255 => Unknown,
            _ => Unknown
        }
    }
}

impl Serializable for KeyPress {
    fn read(sr: &mut SerializationReader) -> SerializationResult<Self> {
        Ok(sr.read::<u8>("KeyPress")?.into())
    }
    fn write(&self, sw:&mut SerializationWriter) { sw.write::<u8>(&(*self as u8)) }
}
