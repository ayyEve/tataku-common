use crate::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct ScoreSubmit {
    pub username: String,
    pub password: String,
    pub game: String,
    pub replay: Replay
}