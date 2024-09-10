use crate::prelude::*;

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
#[derive(Serialize, Deserialize, PacketSerialization)]
#[derive(Reflect)]
#[reflect(from_string = "auto")]
#[Packet(type="u8")]
pub enum LobbyState {
    #[Packet(id=0)]
    Idle,
    
    #[Packet(id=1)]
    Playing,

    #[default]
    #[Packet(id=255)]
    Unknown,
}
