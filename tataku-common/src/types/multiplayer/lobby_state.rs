use crate::prelude::*;

#[repr(u8)]
#[derive(Serialize, Deserialize)]
#[derive(Reflect, PacketSerialization, FromStr)]
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub enum LobbyState {
    #[packet(id=0)] Idle,
    #[packet(id=1)] Playing,
    #[default]
    #[packet(id=255)] Unknown,
}
