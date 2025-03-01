use crate::prelude::*;

#[derive(Copy, Clone, Debug, Default)]
#[derive(PacketSerialization)]
#[packet(type="u8")]
pub enum ServerDropReason {
    /// user logged in from a game somewhere else
    #[packet(id=0)] OtherLogin,

    /// received a bad packet
    #[packet(id=1)] BadPacket,
    
    /// Server is stopping
    #[packet(id=2)] ServerClosing,

    // something else
    #[default]
    #[packet(id=255)] Other,
}
