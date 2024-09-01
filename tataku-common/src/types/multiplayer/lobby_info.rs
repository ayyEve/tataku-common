use crate::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[derive(Serialize, Deserialize, Serializable)]
#[derive(Reflect)]
pub struct LobbyInfo {
    /// lobby id
    pub id: u32,

    /// name of the lobby
    pub name: String,
    
    /// does this lobby have a password
    pub has_password: bool,

    /// who is the current host
    pub host: u32,

    /// ids of the users in this lobby
    pub players: Vec<u32>,

    /// title of the current beatmap
    pub current_beatmap: Option<String>,

    /// current state of the lobby
    pub state: LobbyState,
}


/// extra lobby data
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[derive(Serialize, Deserialize, Serializable)]
#[derive(Reflect)]
pub struct FullLobbyInfo {
    /// lobby id
    pub id: u32,

    /// name of the lobby
    pub name: String,
    
    /// who is the current host
    pub host: u32,
    
    /// current state of the lobby
    pub state: LobbyState,

    /// ids of the users in this lobby
    pub players: Vec<LobbyUser>,

    /// slot states
    pub slots: HashMap<u8, LobbySlot>,

    /// title of the current beatmap
    pub current_beatmap: Option<LobbyBeatmap>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[derive(Serialize, Deserialize, Serializable)]
#[derive(Reflect)]
pub struct LobbyBeatmap {
    pub title: String,
    pub hash: Md5Hash,
    pub mode: String,
    pub map_game: MapGame,
}
