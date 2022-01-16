use crate::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[derive(PacketSerialization)]
#[Packet(type="u8")]
pub enum LoginStatus {
    /// some unknown error occurred
    #[Packet(id=0, default_variant)]
    UnknownError,
    /// login success
    #[Packet(id=1)]
    Ok,
    /// password is incorrect
    #[Packet(id=2)]
    BadPassword,
    /// user doesnt exist
    #[Packet(id=3)]
    NoUser,
}