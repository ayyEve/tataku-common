use crate::macros::*;
use crate::serialization::*;
use crate::packets::PacketId;


#[allow(non_camel_case_types)]
#[derive(PacketSerialization)]
#[derive(Clone, Debug, Default)]
#[packet_type(u8)]
pub enum ChatPacket {
    /// client is sending a message to the server
    #[packet(id=0)]
    Client_SendMessage {
        /// where is this message going?
        /// - non-dms will start with a #
        channel: String,
        /// what does the message contain?
        message: String
    },
    /// server is relaying this message
    #[packet(id=1)]
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
    #[packet(id=2)]
    Client_GetFriends,
    /// server giving the client all the user's friend's ids
    #[packet(id=3)]
    Server_FriendsList {
        friend_ids: Vec<u32>,
    },

    /// client-side changed friend status with someone
    #[packet(id=4)]
    Client_UpdateFriend {
        friend_id: u32,
        /// true if adding friend, false if removing friend
        is_friend: bool,
    },
    /// server-side changed friend status with someone
    /// NOTE: this *WILL* be sent as a response to a Client_UpdateFriend request, however it can also be sent if the user updated the friend status through the web
    #[packet(id=5)]
    Server_UpdateFriend {
        friend_id: u32,
        /// true if adding friend, false if removing friend
        is_friend: bool,
    },

    /// client wants to join a chat channel
    #[packet(id=6)]
    Client_JoinChannel {
        /// name of the channel to join
        channel: String,
        /// password for the channel (empty = no password)
        password: String,
    },

    /// client joined a chat channel
    #[packet(id=7)]
    Server_JoinChannel {
        channel: String,
        previous_messages: Vec<ChatHistoryMessage>
    },


    
    #[default]
    #[packet(id=255)]
    Unknown,
}
impl From<ChatPacket> for PacketId {
    fn from(val: ChatPacket) -> Self {
        PacketId::Chat_Packet { packet: val }
    }
}

#[derive(Serializable)]
#[derive(Default, Clone, Debug)]
pub struct ChatHistoryMessage {
    pub user_id: u32,
    pub username: String,
    /// time in ms since linux epoch
    pub time: u64,
    pub message: String,
}
