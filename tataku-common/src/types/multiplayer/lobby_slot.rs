use crate::prelude::*;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[derive(Serialize, Deserialize)]
#[derive(Reflect, PacketSerialization)]
#[reflect(from_string = "auto")]
#[packet(type="u8")]
pub enum LobbySlot {
    #[default]
    #[packet(id=0)] Empty,
    #[packet(id=1)] Filled {user: u32},
    #[packet(id=2)] Locked,
    #[packet(id=255)] Unknown,
}
impl LobbySlot {
    pub fn is_free(&self) -> bool {
        matches!(self, Self::Empty)
    }
    pub fn is_locked(&self) -> bool {
        matches!(self, Self::Locked)
    }
}
