use crate::prelude::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct ScoreSubmit {
    /// tataku username
    pub username: String,
    /// tataku password
    pub password: String,
    /// not the game of the map, the game thats submitting this score
    pub game: String,

    /// replay data to store on the server
    pub replay: Replay,

    /// info for the map that was just played
    /// this helpers the server get info for this map if it doesnt already have it
    pub map_info: ScoreMapInfo,
}


#[derive(Serialize, Deserialize, Clone)]
pub struct ScoreMapInfo {
    pub game: MapGame,
    pub map_hash: String,
    pub playmode: String,
}


#[derive(Serialize, Deserialize, Clone)]
pub enum MapGame {
    Osu,
    Quaver,
    Other(String),
}


#[derive(Serialize, Deserialize, Clone)]
pub enum SubmitResponse {
    /// score was not submitted, with the following reason (enum, then human readable)
    NotSubmitted(NotSubmittedReason, String),
    Submitted {
        /// tataku id for this score
        score_id: u64,
        /// map placing
        placing: u32,
        /// how much perf was this worth?
        performance_rating: f32,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum NotSubmittedReason {
    InternalError,

    NoUser,
    UserBanned,

    MapNotFound,
    GameNotAccepted,


    Other(String)
}

