use crate::prelude::*;

#[derive(Clone, Debug)]
#[allow(non_camel_case_types)]
#[derive(PacketSerialization)]
#[Packet(type="u8")]
pub enum SpectatorPacket {
    /// client wants to spectate someone
    #[Packet(id=0)]
    Client_Spectate,

    #[Packet(id=1)]
    Server_SpectateResult {
        result: SpectateResult
    },

    /// server telling spectator host someone has spectated them
    #[Packet(id=2)]
    Server_SpectatorJoined {
        /// user id of the new spectator
        user_id: u32,

        /// username of the new spectator
        /// - this is here just to ensure the host has a display name for the spectator
        username: String,
    },

    /// client is no longer spectating the host
    #[Packet(id=3)]
    Client_LeaveSpectator,

    /// server telling us someone stopped spectating
    /// - **NOTE**: if user_id is your own, you stopped spectating
    /// - this can be used to see if you should stop spectating for some reason
    #[Packet(id=4)]
    Server_SpectatorLeft {
        /// which user stopped spectating
        user_id: u32
    },

    /// client is sending us spectator frames
    #[Packet(id=5)]
    Client_SpectatorFrames {
        frames: Vec<SpectatorFrame>
    },

    /// server is sending us spectator frames
    #[Packet(id=6)]
    Server_SpectatorFrames {
        frames: Vec<SpectatorFrame>,
    },
    
    #[Packet(id=255)]
    Unknown,
}

impl SpectatorPacket {
    pub fn with_host(self, host_id: u32) -> PacketId {
        PacketId::Spectator_Packet { host_id, packet: self }
    }
}
