use crate::prelude::*;

#[derive(Clone, Debug)]
#[allow(non_camel_case_types)]
#[derive(PacketSerialization)]
#[Packet(type="u16")]
pub enum PacketId {
    // ======= Unknown =======
    /// we dont know what this packet is.
    /// - if you get this, its probably best to stop reading the current incoming data
    #[Packet(id=0)]
    Unknown, 

    // ======= Ping =======
    /// ping!
    /// - use this if your websocket library does not support native pings
    #[Packet(id=1)]
    Ping,
    /// pong!
    /// - use this if your websocket library does not support native pongs
    #[Packet(id=2)]
    Pong,


    // ======= login/Server things =======

    /// Client wants to log into the server
    #[Packet(id=100)]
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
    #[Packet(id=101)]
    Server_LoginResponse {
        // was the login successful?
        status: LoginStatus,
        // what is your user id?
        user_id: u32,
    },
    #[Packet(id=102)]
    Server_Permissions {
        /// user this permissions packet is assiciated with
        user_id: u32,
        /// the permissions for said user
        /// - **NOTE**!!! this is a vec for rust code only, everywhere else (including when written to the packet) 
        /// this will be a u16!!
        /// - **THIS SHOULD BE READ AS A U16, NOT A LIST OF U16**
        permissions: Vec<ServerPermissions>,
    },
    #[Packet(id=103)]
    Server_UserJoined {
        /// id of the user who joined
        user_id: u32,
        /// username of said user
        username: String,
        /// what game is the user playing?
        game: String,
    },
    #[Packet(id=104)]
    /// client is disconnecting from the server
    Client_LogOut,
    #[Packet(id=105)]
    Server_UserLeft {
        /// id of the user who is leaving
        user_id: u32
    },
    /// server is telling the client something
    #[Packet(id=106)]
    Server_Notification {
        /// the contents of the notification
        message: String,
        /// the severity of the notification
        severity: Severity
    },
    /// server is dropping the connection for some reason
    #[Packet(id=107)]
    Server_DropConnection {
        /// why was the connection dropped?
        reason: ServerDropReason,
        /// text for reason
        message: String
    },
    /// there was an error within spec
    #[Packet(id=108)]
    Server_Error {
        /// what is the reason for the error?
        code: ServerErrorCode,
        /// text representation, provides extra info
        error: String
    },

    
    // ======= Status Updates =======
    #[Packet(id=200)]
    Client_StatusUpdate {
        /// what is the user doing?
        action: UserAction,
        /// is there some text associated with this?
        action_text: String,
        /// what mode is the user in?
        mode: String
    },
    #[Packet(id=201)]
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
    #[Packet(id=202)]
    Client_NotifyScoreUpdate,
    /// contains the info for the above packet
    #[Packet(id=203)]
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
    #[Packet(id=300)]
    Chat_Packet {
        packet: ChatPacket
    },

    // ======= Spectator =======
    #[Packet(id=400)]
    Spectator_Packet {
        /// user id of the host
        host_id: u32,
        /// the actual request
        packet: SpectatorPacket
    },

    // ======= Multiplayer =======
    #[Packet(id=500)]
    Multiplayer_Packet {
        packet: MultiplayerPacket
    }

}
