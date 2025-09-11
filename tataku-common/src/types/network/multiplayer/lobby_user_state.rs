use crate::macros::*;
use crate::reflection::*;
use crate::serialization::*;

#[repr(u8)]
#[derive(Serialize, Deserialize)]
#[derive(Reflect, PacketSerialization, FromStr)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
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
