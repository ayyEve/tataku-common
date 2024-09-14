use crate::prelude::*;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[derive(Serialize, Deserialize, PacketSerialization)]
#[derive(Reflect)]
#[reflect(from_string = "auto")]
#[Packet(type="u8")]
pub enum LobbySlot {
    #[default]
    #[Packet(id=0)]
    Empty,
    
    #[Packet(id=1)]
    Filled {user: u32},

    #[Packet(id=2)]
    Locked,

    #[Packet(id=255)]
    Unknown,
}
impl LobbySlot {
    pub fn is_free(&self) -> bool {
        matches!(self, Self::Empty)
    }
    pub fn is_locked(&self) -> bool {
        matches!(self, Self::Locked)
    }
}
