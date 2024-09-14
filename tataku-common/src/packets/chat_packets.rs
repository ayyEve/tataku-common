use crate::prelude::*;


#[derive(Clone, Debug)]
#[allow(non_camel_case_types)]
#[derive(PacketSerialization)]
#[Packet(type="u8")]
pub enum ChatPacket {
    /// client is sending a message to the server
    #[Packet(id=0)]
    Client_SendMessage {
        /// where is this message going?
        /// - non-dms will start with a #
        channel: String,
        /// what does the message contain?
        message: String
    },
    /// server is relaying this message
    #[Packet(id=1)]
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
    #[Packet(id=2)]
    Client_GetFriends,
    /// server giving the client all the user's friend's ids
    #[Packet(id=303)]
    Server_FriendsList {
        friend_ids: Vec<u32>,
    },

    /// client-side changed friend status with someone
    #[Packet(id=4)]
    Client_UpdateFriend {
        friend_id: u32,
        /// true if adding friend, false if removing friend
        is_friend: bool,
    },
    /// server-side changed friend status with someone
    /// NOTE: this *WILL* be sent as a response to a Client_UpdateFriend request, however it can also be sent if the user updated the friend status through the web
    #[Packet(id=5)]
    Server_UpdateFriend {
        friend_id: u32,
        /// true if adding friend, false if removing friend
        is_friend: bool,
    },

    /// client wants to join a chat channel
    #[Packet(id=6)]
    Client_JoinChannel {
        /// name of the channel to join
        channel: String,
        /// password for the channel (empty = no password)
        password: String,
    },

    /// client joined a chat channel
    #[Packet(id=7)]
    Server_JoinChannel {
        channel: String,
        previous_messages: Vec<ChatHistoryMessage>
    },


    
    #[Packet(id=255)]
    Unknown,
}
impl From<ChatPacket> for PacketId {
    fn from(val: ChatPacket) -> Self {
        PacketId::Chat_Packet { packet: val }
    }
}

#[derive(Default, Clone, Debug, Serializable)]
pub struct ChatHistoryMessage {
    pub user_id: u32,
    pub username: String,
    /// time in ms since linux epoch
    pub time: u64,
    pub message: String,
}
