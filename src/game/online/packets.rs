use crate::game::Serializable;

pub struct Packet {
    pub packet_id: PacketId,
    pub length: u64,
    pub data: Vec<u8>
}


#[derive(Copy, Clone)]
pub enum PacketId {
    Unknown(u16), // read type
    UserJoined,
    UserLeft,
}
impl Into<PacketId> for u16 {
    fn into(self) -> PacketId {
        use PacketId::*;

        match self {
            1 => UserJoined,
            2 => UserLeft,

            n => Unknown(n)
        }
    }
}
impl Into<u16> for PacketId {
    fn into(self) -> u16 {
        use PacketId::*;

        match self {
            UserJoined => 1,
            UserLeft => 2,

            Unknown(n) => n,
        }
    }
}
impl Serializable for PacketId {
    fn read(sr:&mut crate::game::SerializationReader) -> Self {
        sr.read_u16().into()
    }

    fn write(&self, sw:&mut crate::game::SerializationWriter) {
        sw.write_u16(self.clone().into())
    }
}
