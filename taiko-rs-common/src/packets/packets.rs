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
        reason: String
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
        mode: PlayMode
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
        mode: PlayMode
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

    /// client is sending a message to the server
    #[Packet(id=300)]
    Client_SendMessage {
        /// where is this message going?
        /// - non-dms will start with a #
        channel: String,
        /// what does the message contain?
        message: String
    },
    /// server is relaying this message
    #[Packet(id=301)]
    Server_SendMessage {
        /// who sent this message
        sender_id: u32,
        /// where is this message going?
        /// - non-dms will start with a #
        channel: String,
        /// what does the message contain?
        message: String
    },


    // ======= Spectator =======

    /// client wants to spectate someone
    #[Packet(id=400)]
    Client_Spectate {
        /// user id which the user wants to spectate
        host_id: u32
    },
    /// server telling spectator host someone has spectated them
    #[Packet(id=401)]
    Server_SpectatorJoined {
        /// user id of the new spectator
        user_id: u32,

        /// username of the new spectator
        /// - this is here just to ensure the host has a display name for the spectator
        username: String,
    },
    /// client is no longer spectating their host
    #[Packet(id=402)]
    Client_LeaveSpectator,
    /// server telling us someone stopped spectating
    /// - **NOTE**: if user_id is your own, you stopped spectating
    /// - this can be used to see if you should stop spectating for some reason
    #[Packet(id=403)]
    Server_SpectatorLeft {
        /// which user stopped spectating
        user_id: u32
    },
    /// client is sending us spectator frames
    #[Packet(id=404)]
    Client_SpectatorFrames {
        frames: Vec<(f32, SpectatorFrameData)>
    },
    /// server is sending us spectator frames
    #[Packet(id=405)]
    Server_SpectatorFrames {
        frames: Vec<(f32, SpectatorFrameData)>
    },
    /// server is telling us someone wants to know our current in-game progress
    #[Packet(id=406)]
    Server_SpectatorPlayingRequest {
        /// who wants the update?
        user_id: u32
    },

    // ======= Multiplayer? =======
    // 500-599 reserved for multi
}

