use crate::prelude::*;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[derive(Serialize, Deserialize)]
#[derive(Reflect, PacketSerialization)]
#[packet(type="u8")]
#[reflect(from_string = "auto")]
pub enum LobbyUserState {
    #[packet(id=0)] NoMap,
    #[packet(id=1)] InGame,
    #[packet(id=2)] Ready,
    #[packet(id=3)] NotReady,
    #[default]
    #[packet(id=255)] Unknown,
}

impl LobbyUserState {
    /// is it safe to assume this user has the current map?
    pub fn has_map(&self) -> bool {
        !matches!(self, LobbyUserState::NoMap | LobbyUserState::Unknown)
    }
}
