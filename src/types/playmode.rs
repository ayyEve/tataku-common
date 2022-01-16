use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[derive(PacketSerialization)]
#[Packet(type="u8", gen_to_from)]
pub enum PlayMode {
    #[Packet(id=0)]
    Standard,
    #[Packet(id=1)]
    Taiko,
    #[Packet(id=2)]
    Catch,
    #[Packet(id=3)]
    Mania,
    #[Packet(id=4)]
    Adofai,
    #[allow(non_camel_case_types)]
    #[Packet(id=5)]
    pTyping,
    
    #[Packet(id=255)]
    Unknown = 255,
}
impl Default for PlayMode {
    fn default() -> Self {
        PlayMode::Standard
    }
}