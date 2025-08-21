use crate::prelude::*;

#[repr(u8)]
#[derive(PacketSerialization)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum UserAction {
    #[default]
    #[packet(id=0)] Unknown,
    #[packet(id=1)] Idle,
    #[packet(id=2)] Ingame,
    #[packet(id=3)] Leaving,
    #[packet(id=4)] Editing,
}
