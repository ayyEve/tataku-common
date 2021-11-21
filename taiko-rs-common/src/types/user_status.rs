use crate::serialization::Serializable;

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u16)]
pub enum UserAction {
    Unknown = 0,
    Idle,
    Ingame,
    Leaving,
    Editing
}
impl From<u16> for UserAction {
    fn from(n: u16) -> Self {
        use UserAction::*;
        match n {
            1 => Idle,
            2 => Ingame,
            3 => Leaving,
            4 => Editing,
            _=> Unknown
        }
    }
}


impl Serializable for UserAction {
    fn read(sr:&mut crate::serialization::SerializationReader) -> Self {
        sr.read_u16().into()
    }

    fn write(&self, sw:&mut crate::serialization::SerializationWriter) {
        sw.write_u16(*self as u16)
    }
}