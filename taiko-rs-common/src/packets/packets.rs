use std::convert::TryInto;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::serialization::Serializable;

#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
#[allow(non_camel_case_types)]
pub enum PacketId {
    Unknown = 0,
    Client_UserLogin,
    Server_LoginResponse,

    Client_LogOut,
    Server_UserJoined,
    Server_UserLeft,

    Client_StatusUpdate
}
impl PacketId {
    pub fn from(n:u16) -> PacketId {
        n.try_into().unwrap_or(PacketId::Unknown)
    }
}
impl Serializable for PacketId {
    fn read(sr:&mut crate::serialization::SerializationReader) -> Self {
        sr.read_u16().try_into().unwrap_or(PacketId::Unknown)
    }

    fn write(&self, sw:&mut crate::serialization::SerializationWriter) {
        sw.write_u16(self.clone().into())
    }
}

