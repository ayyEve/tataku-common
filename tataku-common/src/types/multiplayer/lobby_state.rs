use crate::prelude::*;

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
#[derive(Serialize, Deserialize)]
#[derive(Reflect, PacketSerialization)]
#[reflect(from_string = "auto")]
#[packet(type="u8")]
pub enum LobbyState {
    #[packet(id=0)] Idle,
    #[packet(id=1)] Playing,
    #[default]
    #[packet(id=255)] Unknown,
}
