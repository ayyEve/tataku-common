use crate::prelude::*;

#[allow(non_camel_case_types)]
#[derive(PacketSerialization)]
#[derive(Clone, Debug, Default)]
#[packet_type(u8)]
pub enum SpectatorPacket {
    /// client wants to spectate someone
    #[packet(id=0)]
    Client_Spectate,

    #[packet(id=1)]
    Server_SpectateResult {
        result: SpectateResult
    },

    /// server telling spectator host someone has spectated them
    #[packet(id=2)]
    Server_SpectatorJoined {
        /// user id of the new spectator
        user_id: u32,

        /// username of the new spectator
        /// - this is here just to ensure the host has a display name for the spectator
        username: String,
    },

    /// client is no longer spectating the host
    #[packet(id=3)]
    Client_LeaveSpectator,

    /// server telling us someone stopped spectating
    /// - **NOTE**: if user_id is your own, you stopped spectating
    /// - this can be used to see if you should stop spectating for some reason
    #[packet(id=4)]
    Server_SpectatorLeft {
        /// which user stopped spectating
        user_id: u32
    },

    /// client is sending us spectator frames
    #[packet(id=5)]
    Client_SpectatorFrames {
        frames: Vec<SpectatorFrame>
    },

    /// server is sending us spectator frames
    #[packet(id=6)]
    Server_SpectatorFrames {
        frames: Vec<SpectatorFrame>,
    },
    
    #[default]
    #[packet(id=255)]
    Unknown,
}

impl SpectatorPacket {
    pub fn with_host(self, host_id: u32) -> PacketId {
        PacketId::Spectator_Packet { host_id, packet: self }
    }
}
