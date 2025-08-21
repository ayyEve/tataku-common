use crate::prelude::*;

#[repr(u8)]
#[derive(PacketSerialization)]
#[derive(Copy, Clone, Debug, Default)]
pub enum Severity {
    #[default]
    #[packet(id=0)] Info,
    #[packet(id=1)] Warning,
    #[packet(id=2)] Error,
}
