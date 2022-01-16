use crate::prelude::*;

#[derive(Copy, Clone, Debug)]
#[derive(PacketSerialization)]
#[Packet(type="u8")]
pub enum ServerErrorCode {
    #[Packet(id=0)]
    Unknown,
    #[Packet(id=1)]
    CantSpectate
}