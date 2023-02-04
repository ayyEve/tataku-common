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
impl Serializable for ScoreSubmit {
    fn read(sr:&mut SerializationReader) -> SerializationResult<Self> where Self: Sized {
        Ok(Self {
            username: sr.read()?,
            password: sr.read()?,
            game: sr.read()?,
            replay: sr.read()?,
            map_info: sr.read()?,
        })
    }

    fn write(&self, sw:&mut SerializationWriter) {
        sw.write(&self.username);
        sw.write(&self.password);
        sw.write(&self.game);
        sw.write(&self.replay);
        sw.write(&self.map_info);
    }
}


#[derive(Serialize, Deserialize, Clone)]
pub struct ScoreMapInfo {
    pub game: MapGame,
    pub map_hash: String,
    pub playmode: String,
}
impl Serializable for ScoreMapInfo {
    fn read(sr:&mut SerializationReader) -> SerializationResult<Self> where Self: Sized {
        Ok(Self {
            game: sr.read()?,
            map_hash: sr.read()?,
            playmode: sr.read()?
        })
    }

    fn write(&self, sw:&mut SerializationWriter) {
        sw.write(&self.game);
        sw.write(&self.map_hash);
        sw.write(&self.playmode);
    }
}


#[derive(Serialize, Deserialize, Clone)]
pub enum MapGame {
    Osu,
    Quaver,
    Other(String),
}
impl Serializable for MapGame {
    fn read(sr:&mut SerializationReader) -> SerializationResult<Self> where Self: Sized {
        let s:String = sr.read()?;
        match &*s.to_lowercase() {
            "osu" => Ok(Self::Osu),
            "quaver" => Ok(Self::Quaver),
            _ => Ok(Self::Other(s)),
        }
    }

    fn write(&self, sw:&mut SerializationWriter) {
        match self {
            MapGame::Osu => sw.write(&"osu".to_owned()),
            MapGame::Quaver => sw.write(&"quaver".to_owned()),
            MapGame::Other(s) => sw.write(s),
        }
    }
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

