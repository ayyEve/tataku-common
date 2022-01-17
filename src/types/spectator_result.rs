use crate::prelude::*;

#[allow(non_camel_case_types)]
#[derive(PacketSerialization, Copy, Clone, Debug)]
#[Packet(type="u8")]
pub enum SpectateResult {
    #[Packet(id=0)]
    Ok,

    #[Packet(id=1)]
    Error_SpectatingBot,

    #[Packet(id=2)]
    Error_HostOffline,

    #[Packet(id=255, default_variant)]
    Error_Unknown,
}