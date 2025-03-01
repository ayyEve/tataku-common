use crate::prelude::*;

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct ScoreSubmit {
    /// tataku username
    pub username: String,
    
    /// tataku password
    pub password: String,

    /// not the game of the map, the game thats submitting this score
    pub game: String,

    /// score data to store on the server
    pub score: Score,

    /// info for the map that was just played
    /// this helpers the server get info for this map if it doesnt already have it
    pub map_info: ScoreMapInfo,
}
impl Serializable for ScoreSubmit {
    fn read(sr: &mut SerializationReader) -> SerializationResult<Self> where Self: Sized {
        sr.push_parent("Score Submit");
        let a = Ok(Self {
            username: sr.read("username")?,
            password: sr.read("password")?,
            game: sr.read("game")?,
            score: sr.read("score")?,
            map_info: sr.read("map_info")?,
        });
        sr.pop_parent();
        a
    }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write(&self.username);
        sw.write(&self.password);
        sw.write(&self.game);
        sw.write(&self.score);
        sw.write(&self.map_info);
    }
}


#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct ScoreMapInfo {
    pub game: MapGame,
    pub map_hash: Md5Hash,
    pub playmode: String,
}
impl Serializable for ScoreMapInfo {
    fn read(sr: &mut SerializationReader) -> SerializationResult<Self> where Self: Sized {
        sr.push_parent("ScoreMapInfo");
        let a = Ok(Self {
            game: sr.read("game")?,
            map_hash: sr.read("map_hash")?,
            playmode: sr.read("playmode")?
        });

        sr.pop_parent();
        a
    }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write(&self.game);
        sw.write(&self.map_hash);
        sw.write(&self.playmode);
    }
}


#[derive(Clone)]
#[derive(Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum NotSubmittedReason {
    InternalError,

    NoUser,
    UserBanned,

    MapNotFound,
    GameNotAccepted,

    Other(String)
}
