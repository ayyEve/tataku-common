use crate::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[derive(PacketSerialization)]
#[packet(type="u8")]
pub enum UserAction {
    #[default]
    #[packet(id=0)] Unknown,
    #[packet(id=1)] Idle,
    #[packet(id=2)] Ingame,
    #[packet(id=3)] Leaving,
    #[packet(id=4)] Editing,
}
