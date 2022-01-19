use crate::prelude::*;

#[derive(PacketSerialization, Copy, Clone, Debug)]
#[Packet(type="u8")]
pub enum ServerDropReason {
    /// user logged in from a game somewhere else
    #[Packet(id=0)]
    OtherLogin,

    /// received a bad packet
    #[Packet(id=1)]
    BadPacket,

    // something else
    #[Packet(id=255, default_variant)]
    Other,
}