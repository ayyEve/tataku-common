use crate::prelude::*;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Default)]
#[derive(PacketSerialization)]
#[packet(type="u8")]
pub enum SpectateResult {
    /// spectate request was accepted
    #[packet(id=0)] Ok,

    /// trying to spectate a bot
    #[packet(id=1)] Error_SpectatingBot,

    /// user you're trying to spec doesnt exist or is offline
    #[packet(id=2)] Error_HostOffline,

    /// you're trying to spectate yourself
    #[packet(id=3)] Error_SpectatingYourself,

    /// some other error
    #[default]
    #[packet(id=255)] Error_Unknown,
}