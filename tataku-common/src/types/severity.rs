use crate::prelude::*;

#[derive(Copy, Clone, Debug, Default)]
#[derive(PacketSerialization)]
#[packet(type="u8")]
pub enum Severity {
    #[default]
    #[packet(id=0, default_variant)] Info,
    #[packet(id=1)] Warning,
    #[packet(id=2)] Error,
}
