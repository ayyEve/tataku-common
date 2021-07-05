use std::convert::TryInto;

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::serialization::Serializable;

#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive, PartialEq)]
#[repr(u16)]
pub enum UserAction {
    Unknown = 0,
    Idle,
    Ingame,
    Leaving,
}


impl Serializable for UserAction {
    fn read(sr:&mut crate::serialization::SerializationReader) -> Self {
        sr.read_u16().try_into().unwrap_or(UserAction::Unknown)
    }

    fn write(&self, sw:&mut crate::serialization::SerializationWriter) {
        sw.write_u16(self.clone().into())
    }
}