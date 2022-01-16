use crate::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
#[derive(PacketSerialization)]
#[Packet(type="u8")]
pub enum UserAction {
    #[Packet(id=0, default_variant)]
    Unknown,
    #[Packet(id=1)]
    Idle,
    #[Packet(id=2)]
    Ingame,
    #[Packet(id=3)]
    Leaving,
    #[Packet(id=4)]
    Editing
}