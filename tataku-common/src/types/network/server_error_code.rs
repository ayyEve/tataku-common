use crate::serialization::*;

#[repr(u8)]
#[derive(crate::macros::PacketSerialization)]
#[derive(Copy, Clone, Debug, Default)]
pub enum ServerErrorCode {
    #[default]
    #[packet(id=0)] Unknown,
    #[packet(id=1)] CantSpectate
}
