use crate::prelude::*;

#[derive(Clone, Debug)]
#[derive(PacketSerialization)]
#[Packet(type="u8")]
pub enum ServerErrorCode {
    #[Packet(id=0)]
    Unknown,
    #[Packet(id=1)]
    CantSpectate
}