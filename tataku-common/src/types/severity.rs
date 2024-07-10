use crate::prelude::*;

#[derive(Copy, Clone, Debug)]
#[derive(PacketSerialization)]
#[Packet(type="u8")]
pub enum Severity {
    #[Packet(id=0, default_variant)]
    Info,
    #[Packet(id=1)]
    Warning,
    #[Packet(id=2)]
    Error,
}