use std::convert::TryInto;
use crate::serialization::Serializable;

#[derive(Copy, Clone, Debug)]
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
    /// sender_id, channel_id, message
    Client_SendMessage, 
    /// sender_id, channel_id, message
    Server_SendMessage, 

    // spectator?
    /// client wants to spectate someone
    Client_Spectate, // user_id to spectate
    Server_SpectatorJoined, // user_id of spectator who joined

    Client_SpectatorLeft,
    /// user_id of spectator who left
    /// if user_id is your own, you stopped spectating
    Server_SpectatorLeft, 

    Client_SpectatorFrames, // frame_count, [SpectatorFrame]
    Server_SpectatorFrames, // host_id, frame_count, [SpectatorFrame]


    Ping,
    Pong,

    // multiplayer?
}
impl PacketId {
    pub fn from(n:u16) -> PacketId {
        n.try_into().unwrap_or(PacketId::Unknown)
    }
}
impl From<u16> for PacketId {
    fn from(n: u16) -> Self {
        use PacketId::*;
        
        match n {
            // login
            1 => Client_UserLogin,
            2 => Server_LoginResponse,

            // status updates
            3 => Client_StatusUpdate,
            4 => Server_UserStatusUpdate,
            5 => Client_NotifyScoreUpdate,
            6 => Server_ScoreUpdate,
            7 => Client_LogOut,
            8 => Server_UserJoined,
            9 => Server_UserLeft,

            // chat
            10 => Client_SendMessage,
            11 => Server_SendMessage,

            // spectator
            12 => Client_Spectate,
            13 => Server_SpectatorJoined,
            14 => Client_SpectatorLeft,
            15 => Server_SpectatorLeft,
            16 => Client_SpectatorFrames,
            17 => Server_SpectatorFrames,

            // ping/ping (i hate that these are here and not ids 1 and 2)
            18 => Ping,
            19 => Pong,

            // multiplayer

            _ => Unknown
        }
    }
}

impl Serializable for PacketId {
    fn read(sr:&mut crate::serialization::SerializationReader) -> Self {
        sr.read_u16().into()
    }

    fn write(&self, sw:&mut crate::serialization::SerializationWriter) {
        sw.write_u16(*self as u16)
    }
}
