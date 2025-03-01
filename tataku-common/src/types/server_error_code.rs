use crate::prelude::*;

#[derive(Copy, Clone, Debug, Default)]
#[derive(PacketSerialization)]
#[packet(type="u8")]
pub enum ServerErrorCode {
    #[default]
    #[packet(id=0)] Unknown,
    #[packet(id=1)] CantSpectate
}
