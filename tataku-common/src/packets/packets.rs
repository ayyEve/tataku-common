use crate::prelude::*;

#[allow(clippy::large_enum_variant)]
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Default)]
#[derive(PacketSerialization)]
#[packet(type="u16")]
// #[packet(extra_logging)]
pub enum PacketId {
    // ======= Unknown =======
    /// we dont know what this packet is.
    /// - if you get this, its probably best to stop reading the current incoming data
    #[default]
    #[packet(id=0)]
    Unknown, 

    // ======= Ping =======
    /// ping!
    /// - use this if your websocket library does not support native pings
    #[packet(id=1)]
    Ping,
    /// pong!
    /// - use this if your websocket library does not support native pongs
    #[packet(id=2)]
    Pong,


    // ======= login/Server things =======

    /// Client wants to log into the server
    #[packet(id=100)]
    Client_UserLogin {
        /// which version of the packet protocol does this client use?
        /// - this will help with future expandability
        protocol_version: u16,
        /// user username
        username: String,
        /// user password
        password: String,
        /// \[game_name]\n\[version_info]
        game: String,
    },
    /// server is telling the client if the login worked
    #[packet(id=101)]
    Server_LoginResponse {
        // was the login successful?
        status: LoginStatus,
        // what is your user id?
        user_id: u32,
    },
    #[packet(id=102)]
    Server_Permissions {
        /// user this permissions packet is assiciated with
        user_id: u32,
        /// the permissions for said user
        /// - **NOTE**!!! this is a vec for rust code only, everywhere else (including when written to the packet) 
        /// 
        /// this will be a u16!!
        /// - **THIS SHOULD BE READ AS A U16, NOT A LIST OF U16**
        permissions: Vec<ServerPermissions>,
    },
    #[packet(id=103)]
    Server_UserJoined {
        /// id of the user who joined
        user_id: u32,
        /// username of said user
        username: String,
        /// what game is the user playing?
        game: String,
    },
    #[packet(id=104)]
    /// client is disconnecting from the server
    Client_LogOut,
    #[packet(id=105)]
    Server_UserLeft {
        /// id of the user who is leaving
        user_id: u32
    },
    /// server is telling the client something
    #[packet(id=106)]
    Server_Notification {
        /// the contents of the notification
        message: String,
        /// the severity of the notification
        severity: Severity
    },
    /// server is dropping the connection for some reason
    #[packet(id=107)]
    Server_DropConnection {
        /// why was the connection dropped?
        reason: ServerDropReason,
        /// text for reason
        message: String
    },
    /// there was an error within spec
    #[packet(id=108)]
    Server_Error {
        /// what is the reason for the error?
        code: ServerErrorCode,
        /// text representation, provides extra info
        error: String
    },

    
    // ======= Status Updates =======
    #[packet(id=200)]
    Client_StatusUpdate {
        /// what is the user doing?
        action: UserAction,
        /// is there some text associated with this?
        action_text: String,
        /// what mode is the user in?
        mode: String
    },
    #[packet(id=201)]
    Server_UserStatusUpdate {
        /// what user is this update for?
        user_id: u32,
        /// what is the user doing?
        action: UserAction,
        /// is there some text associated with this?
        action_text: String,
        /// what mode is the user in?
        mode: String
    },
    /// Sent by a client to notify the server to update their score for everyone
    #[packet(id=202)]
    Client_NotifyScoreUpdate,
    /// contains the info for the above packet
    #[packet(id=203)]
    Server_ScoreUpdate {
        /// user id this score update is for
        user_id: u32,
        // rest of these should be self-explanitory
        total_score: i64,
        ranked_score: i64,
        accuracy: f64,
        play_count: i32,
        rank: i32
    },
 

    // ======= Chat =======
    #[packet(id=300)]
    Chat_Packet {
        packet: ChatPacket
    },

    // ======= Spectator =======
    #[packet(id=400)]
    Spectator_Packet {
        /// user id of the host
        host_id: u32,
        /// the actual request
        packet: SpectatorPacket
    },

    // ======= Multiplayer =======
    #[packet(id=500)]
    Multiplayer_Packet {
        packet: MultiplayerPacket
    }

}
