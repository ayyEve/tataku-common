use crate::prelude::*;

#[derive(Clone, Debug)]
#[allow(non_camel_case_types)]
#[derive(PacketSerialization)]
#[Packet(type="u8")]
pub enum MultiplayerPacket {
    /// client request to get a list of available lobbies
    #[Packet(id=0)]
    Client_LobbyList,

    /// server response with lobby list
    #[Packet(id=1)]
    Server_LobbyList {
        lobbies: Vec<LobbyInfo>
    },

    /// client request to create a lobby
    #[Packet(id=2)]
    Client_CreateLobby {
        /// name of the lobby
        name: String,

        /// lobby password (if empty, no password)
        password: String,

        /// is this a secret lobby? (invite only, not listed in lobby list)
        private: bool,

        /// how many players does the lobby allow?
        players: u8,
    },

    /// server response to lobby creation
    #[Packet(id=3)]
    Server_CreateLobby {
        /// was the lobby creation successful?
        success: bool,
        /// new lobby (if successful)
        lobby: Option<FullLobbyInfo>,
    },

    /// client inviting another user to the lobby
    #[Packet(id=4)]
    Client_LobbyInvite {
        /// invited user id 
        user_id: u32,
    },
    /// server forwarding an invite to a user
    #[Packet(id=5)]
    Server_LobbyInvite {
        /// id of the user who invited this user
        inviter_id: u32,
        /// lobby metadata
        lobby: LobbyInfo,
    },
    
    /// client telling server it wants to know about lobby updates
    #[Packet(id=6)]
    Client_AddLobbyListener,

    /// client telling server it no longer wants to know about lobby updates
    #[Packet(id=7)]
    Client_RemoveLobbyListener,
    

    /// server letting clients know a lobby was created
    /// 
    /// only sent to listening users
    #[Packet(id=8)]
    Server_LobbyCreated {
        lobby: LobbyInfo,
    },
    /// server letting clients know a lobby was deleted
    /// 
    /// only sent to listening users
    #[Packet(id=9)]
    Server_LobbyDeleted {
        lobby_id: u32,
    },
    

    /// client wants to join a lobby
    #[Packet(id=10)]
    Client_JoinLobby {
        lobby_id: u32,
        password: String
    },
    /// server letting the client know if it joined successfully
    #[Packet(id=11)]
    Server_JoinLobby {
        success: bool,
        lobby: Option<FullLobbyInfo>,
    },

    /// server letting clients know a user joined a lobby
    /// 
    /// also sent to listening users
    #[Packet(id=12)]
    Server_LobbyUserJoined {
        lobby_id: u32,
        user_id: u32,
    },

    /// client has left the lobby
    #[Packet(id=13)]
    Client_LeaveLobby,

    /// server letting clients know a user left a lobby
    /// if you received this without yourself leaving the lobby, you were kicked
    /// 
    /// also sent to listening users
    #[Packet(id=14)]
    Server_LobbyUserLeft {
        lobby_id: u32,
        user_id: u32,
    },

    /// server letting clients know a lobby's state has changed
    /// 
    /// also sent to listening users
    #[Packet(id=15)]
    Server_LobbyStateChange {
        lobby_id: u32,
        new_state: LobbyState,
    },


    /// host has changed the beatmap
    #[Packet(id=16)]
    Client_LobbyMapChange {
        new_map: LobbyBeatmap,
    },
    /// server letting clients know a lobby's map has changed
    /// 
    /// also sent to listening users
    #[Packet(id=17)]
    Server_LobbyMapChange {
        lobby_id: u32,
        new_map: LobbyBeatmap,
    },

    /// client changed the state of a slot
    #[Packet(id=18)]
    Client_LobbySlotChange {
        slot: u8,
        new_status: LobbySlot
    },
    /// server changed the state of a slot
    #[Packet(id=19)]
    Server_LobbySlotChange {
        slot: u8,
        new_status: LobbySlot
    },


    /// client's user state has changed
    #[Packet(id=20)]
    Client_LobbyUserState {
        new_state: LobbyUserState
    },
    /// a client's user state has changed
    #[Packet(id=21)]
    Server_LobbyUserState {
        user_id: u32,
        new_state: LobbyUserState
    },



    /// client has changed their mods
    #[Packet(id=22)]
    Client_LobbyUserModsChanged {
        mods: HashSet<String>,
        speed: u16,
    },
    /// a user has changed their mods
    #[Packet(id=23)]
    Server_LobbyUserModsChanged {
        user_id: u32,
        mods: HashSet<String>,
        speed: u16,
    },
    /// the host has changed the lobby's mods
    #[Packet(id=24)]
    Server_LobbyModsChanged {
        /// can user's set their own mods?
        free_mods: bool,
        mods: HashSet<String>,
        speed: u16,
    },

    /// host is assigning a new host
    #[Packet(id=25)]
    Client_LobbyChangeHost {
        new_host: u32,
    },
    /// server is setting the host
    #[Packet(id=26)]
    Server_LobbyChangeHost {
        new_host: u32,
    },


    /// host is requesting map start
    #[Packet(id=27)]
    Client_LobbyStart,
    /// server is requesting clients load the map
    #[Packet(id=28)]
    Server_LobbyStart,

    /// client is letting server know the map is loaded
    #[Packet(id=29)]
    Client_LobbyMapLoaded,
    /// server is telling clients they can begin playing the map
    #[Packet(id=30)]
    Server_LobbyBeginRound,

    /// client sending a score update to the server
    #[Packet(id=31)]
    Client_LobbyScoreUpdate {
        score: Score,
    },
    /// server forwarding a client's score to all clients
    #[Packet(id=32)]
    Server_LobbyScoreUpdate {
        user_id: u32,
        score: Score,
    },

    /// client telling server its completed the map
    #[Packet(id=33)]
    Client_LobbyMapComplete {
        score: Score
    },
    /// server telling clients that a client has completed the map
    #[Packet(id=34)]
    Server_LobbyPlayerMapComplete {
        user_id: u32,
        score: Score
    },
    /// server telling clients that the round is complete
    #[Packet(id=35)]
    Server_LobbyRoundComplete,
    
    #[Packet(id=255)]
    Unknown,
}

impl Into<PacketId> for MultiplayerPacket {
    fn into(self) -> PacketId {
        PacketId::Multiplayer_Packet { packet: self }
    }
}
