use std::convert::TryInto;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::serialization::Serializable;

#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
#[allow(non_camel_case_types)]
pub enum PacketId {
    Unknown = 0,

    // login
    Client_UserLogin,
    Server_LoginResponse,

    // status updates
    Client_StatusUpdate,
    Server_UserStatusUpdate,
    Client_NotifyScoreUpdate,
    Server_ScoreUpdate,
    Client_LogOut,
    Server_UserJoined,
    Server_UserLeft,

    // chat
    Client_SendMessage, // sender_id, channel_id, message
    Server_SendMessage, // sender_id, channel_id, message

    // spectator?
    Client_Spectate, // user_id to spectate
    Server_SpectatorJoined, // user_id of spectator
    Client_SpectatorFrames, // frame_count, [SpectatorFrame]
    Server_SpectatorFrames, // sender_id, frame_count, [SpectatorFrame]

    // multiplayer?
}
impl PacketId {
    pub fn from(n:u16) -> PacketId {
        n.try_into().unwrap_or(PacketId::Unknown)
    }
}
impl Serializable for PacketId {
    fn read(sr:&mut crate::serialization::SerializationReader) -> Self {
        sr.read_u16().try_into().unwrap_or(PacketId::Unknown)
    }

    fn write(&self, sw:&mut crate::serialization::SerializationWriter) {
        sw.write_u16(self.clone().into())
    }
}
