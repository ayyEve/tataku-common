use crate::prelude::*;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[derive(Serialize, Deserialize, PacketSerialization)]
#[derive(Reflect)]
#[Packet(type="u8")]
#[reflect(from_string = "auto")]
pub enum LobbyUserState {
    #[default]
    #[Packet(id=0)]
    NoMap,

    #[Packet(id=1)]
    InGame,
    
    #[Packet(id=2)]
    Ready,

    #[Packet(id=3)]
    NotReady,

    #[Packet(id=255)]
    Unknown,
}

impl LobbyUserState {
    /// is it safe to assume this user has the current map?
    pub fn has_map(&self) -> bool {
        !matches!(self, LobbyUserState::NoMap | LobbyUserState::Unknown)
    }
}
