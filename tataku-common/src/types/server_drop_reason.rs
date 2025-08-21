use crate::prelude::*;

#[repr(u8)]
#[derive(PacketSerialization)]
#[derive(Copy, Clone, Debug, Default)]
pub enum ServerDropReason {
    /// User logged in from a game somewhere else
    #[packet(id=0)] OtherLogin,

    /// Received a bad packet
    #[packet(id=1)] BadPacket,
    
    /// Server is stopping
    #[packet(id=2)] ServerClosing,

    // Something else
    #[default]
    #[packet(id=255)] Other,
}
