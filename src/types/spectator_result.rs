use crate::prelude::*;

#[allow(non_camel_case_types)]
#[derive(PacketSerialization, Copy, Clone, Debug)]
#[Packet(type="u8")]
pub enum SpectateResult {
    /// spectate request was accepted
    #[Packet(id=0)]
    Ok,

    /// trying to spectate a bot
    #[Packet(id=1)]
    Error_SpectatingBot,

    /// user you're trying to spec doesnt exist or is offline
    #[Packet(id=2)]
    Error_HostOffline,

    /// you're trying to spectate yourself
    #[Packet(id=3)]
    Error_SpectatingYourself,

    /// some other error
    #[Packet(id=255, default_variant)]
    Error_Unknown,
}