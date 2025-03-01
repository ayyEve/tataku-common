use crate::prelude::*;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[derive(PacketSerialization)]
#[packet(type="u8")]
pub enum LoginStatus {
    /// some unknown error occurred
    #[default]
    #[packet(id=0)] UnknownError,

    /// login success
    #[packet(id=1)] Ok,

    /// password is incorrect
    #[packet(id=2)] BadPassword,

    /// user doesnt exist
    #[packet(id=3)] NoUser,
    
    /// account has not been activated
    #[packet(id=4)] NotActivated,
}
