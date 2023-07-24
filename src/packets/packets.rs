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

    /// client requesting all friends from the server
    #[Packet(id=302)]
    Client_GetFriends,
    /// server giving the client all the user's friend's ids
    #[Packet(id=303)]
    Server_FriendsList {
        friend_ids: Vec<u32>,
    },

    /// client-side changed friend status with someone
    #[Packet(id=304)]
    Client_UpdateFriend {
        friend_id: u32,
        /// true if adding friend, false if removing friend
        is_friend: bool,
    },
    /// server-side changed friend status with someone
    /// NOTE: this *WILL* be sent as a response to a Client_UpdateFriend request, however it can also be sent if the user updated the friend status through the web
    #[Packet(id=305)]
    Server_UpdateFriend {
        friend_id: u32,
        /// true if adding friend, false if removing friend
        is_friend: bool,
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
    #[Packet(id=407)]
    Server_SpectateResult {
        result: SpectateResult,
        host_id: u32
    },

    // ======= Multiplayer? =======
    // 500-599 reserved for multi

    /// client request to get a list of available lobbies
    #[Packet(id=500)]
    Client_LobbyList,

    /// server response with lobby list
    #[Packet(id=501)]
    Server_LobbyList {
        lobbies: Vec<LobbyInfo>
    },

    /// client request to create a lobby
    #[Packet(id=502)]
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
    #[Packet(id=503)]
    Server_CreateLobby {
        /// was the lobby creation successful?
        success: bool,
        /// new lobby (if successful)
        lobby: Option<FullLobbyInfo>,
    },

    /// client inviting another user to the lobby
    #[Packet(id=504)]
    Client_LobbyInvite {
        /// invited user id 
        user_id: u32,
    },
    /// server forwarding an invite to a user
    #[Packet(id=505)]
    Server_LobbyInvite {
        /// id of the user who invited this user
        inviter_id: u32,
        /// lobby metadata
        lobby: LobbyInfo,
    },
    
    /// client telling server it wants to know about lobby updates
    #[Packet(id=506)]
    Client_AddLobbyListener,

    /// client telling server it no longer wants to know about lobby updates
    #[Packet(id=507)]
    Client_RemoveLobbyListener,
    

    /// server letting clients know a lobby was created
    /// 
    /// only sent to listening users
    #[Packet(id=508)]
    Server_LobbyCreated {
        lobby: LobbyInfo,
    },
    /// server letting clients know a lobby was deleted
    /// 
    /// only sent to listening users
    #[Packet(id=509)]
    Server_LobbyDeleted {
        lobby_id: u32,
    },
    

    /// client wants to join a lobby
    #[Packet(id=510)]
    Client_JoinLobby {
        lobby_id: u32,
        password: String
    },
    /// server letting the client know if it joined successfully
    #[Packet(id=511)]
    Server_JoinLobby {
        success: bool,
        lobby: Option<FullLobbyInfo>,
    },

    /// server letting clients know a user joined a lobby
    /// 
    /// also sent to listening users
    #[Packet(id=512)]
    Server_LobbyUserJoined {
        lobby_id: u32,
        user_id: u32,
    },

    /// client has left the lobby
    #[Packet(id=513)]
    Client_LeaveLobby,

    /// server letting clients know a user left a lobby
    /// if you received this without yourself leaving the lobby, you were kicked
    /// 
    /// also sent to listening users
    #[Packet(id=514)]
    Server_LobbyUserLeft {
        lobby_id: u32,
        user_id: u32,
    },

    /// server letting clients know a lobby's state has changed
    /// 
    /// also sent to listening users
    #[Packet(id=515)]
    Server_LobbyStateChange {
        lobby_id: u32,
        new_state: LobbyState,
    },


    /// host has changed the beatmap
    #[Packet(id=516)]
    Client_LobbyMapChange {
        new_map: LobbyBeatmap,
    },
    /// server letting clients know a lobby's map has changed
    /// 
    /// also sent to listening users
    #[Packet(id=517)]
    Server_LobbyMapChange {
        lobby_id: u32,
        new_map: LobbyBeatmap,
    },

    /// client changed the state of a slot
    #[Packet(id=518)]
    Client_LobbySlotChange {
        slot: u8,
        new_status: LobbySlot
    },
    /// server changed the state of a slot
    #[Packet(id=519)]
    Server_LobbySlotChange {
        slot: u8,
        new_status: LobbySlot
    },


    /// client's user state has changed
    #[Packet(id=520)]
    Client_LobbyUserState {
        new_state: LobbyUserState
    },
    /// a client's user state has changed
    #[Packet(id=521)]
    Server_LobbyUserState {
        user_id: u32,
        new_state: LobbyUserState
    },



    /// client has changed their mods
    #[Packet(id=522)]
    Client_LobbyUserModsChanged {
        mods: HashSet<String>,
        speed: u16,
    },
    /// a user has changed their mods
    #[Packet(id=523)]
    Server_LobbyUserModsChanged {
        user_id: u32,
        mods: HashSet<String>,
        speed: u16,
    },
    /// the host has changed the lobby's mods
    #[Packet(id=524)]
    Server_LobbyModsChanged {
        /// can user's set their own mods?
        free_mods: bool,
        mods: HashSet<String>,
        speed: u16,
    },

    /// host is assigning a new host
    #[Packet(id=525)]
    Client_LobbyChangeHost {
        new_host: u32,
    },
    /// server is setting the host
    #[Packet(id=526)]
    Server_LobbyChangeHost {
        new_host: u32,
    },


    /// host is requesting map start
    #[Packet(id=527)]
    Client_LobbyStart,
    /// server is requesting clients load the map
    #[Packet(id=528)]
    Server_LobbyStart,

    /// client is letting server know the map is loaded
    #[Packet(id=529)]
    Client_LobbyMapLoaded,
    /// server is telling clients they can begin playing the map
    #[Packet(id=530)]
    Server_LobbyBeginRound,

    /// client sending a score update to the server
    #[Packet(id=531)]
    Client_LobbyScoreUpdate {
        score: Score,
    },
    /// server forwarding a client's score to all clients
    #[Packet(id=532)]
    Server_LobbyScoreUpdate {
        user_id: u32,
        score: Score,
    },

    /// client telling server its completed the map
    #[Packet(id=533)]
    Client_LobbyMapComplete {
        score: Score
    },
    /// server telling clients that a client has completed the map
    #[Packet(id=534)]
    Server_LobbyPlayerMapComplete {
        user_id: u32,
        score: Score
    },
    /// server telling clients that the round is complete
    #[Packet(id=535)]
    Server_LobbyRoundComplete,


}

