use crate::{serialization::Serializable, ReplayFrame, PlayMode};
use super::Score;

pub type SpectatorFrame = (u32, SpectatorFrameData);
pub type SpectatorFrames = Vec<SpectatorFrame>;


macro_rules! write {
    ($sw:expr, $($item:expr),+) => {
        $(
            $sw.write($item);
        )+
    };
}

#[derive(Clone, Debug)]
pub enum SpectatorFrameData {
    /// host started a new map
    Play {beatmap_hash:String, mode: PlayMode, mods: String},
    /// host paused current map
    Pause,
    // host unpaused the current map
    UnPause,
    /// host stopped playing
    Stop,
    /// indicates the last time in a packet, so we know where we have data up to.
    /// should probably be renamed but whatever
    Buffer,
    /// host started spectating someone else. deal with this later
    SpectatingOther {user_id:u32},
    /// host pressed a game key
    ReplayFrame {frame:ReplayFrame},

    /// clear up any score innaccuracies, or update new specs with this
    ScoreSync {score:Score},

    /// host is changing the map
    ChangingMap,

    /// response for current map info
    PlayingResponse {user_id:u32, beatmap_hash:String, mode:PlayMode, mods:String, current_time:f32}
}
impl Serializable for SpectatorFrameData {
    fn read(sr:&mut crate::serialization::SerializationReader) -> Self {
        match sr.read_u8() {
            // play
            0 => SpectatorFrameData::Play {beatmap_hash: sr.read_string(), mode: sr.read(), mods: sr.read()},
            // pause
            1 => SpectatorFrameData::Pause,
            // unpause
            2 => SpectatorFrameData::UnPause,
            // stop
            3 => SpectatorFrameData::Stop,
            // buffer
            4 => SpectatorFrameData::Buffer,
            // spectate other
            5 => SpectatorFrameData::SpectatingOther {user_id: sr.read()},
            // key_press
            6 => SpectatorFrameData::ReplayFrame {frame:sr.read()},
            // score sync
            8 => SpectatorFrameData::ScoreSync {score:sr.read()},
            // host changing map
            9 => SpectatorFrameData::ChangingMap,

            // responding with info
            10 => SpectatorFrameData::PlayingResponse {user_id:sr.read(), beatmap_hash:sr.read(), mode:sr.read(), mods:sr.read(), current_time:sr.read()},

            // unknown
            n => panic!("[Spectator] unknown SpectatorFrameData num: {}", n), // unknown
        }
    }

    fn write(&self, sw:&mut crate::serialization::SerializationWriter) {
        match &self {
            SpectatorFrameData::Play {beatmap_hash, mode, mods} => {sw.write_u8(0); sw.write(beatmap_hash.clone()); sw.write(*mode); sw.write(mods.clone())},
            SpectatorFrameData::Pause => sw.write_u8(1),
            SpectatorFrameData::UnPause => sw.write_u8(2),
            SpectatorFrameData::Stop => sw.write_u8(3),
            SpectatorFrameData::Buffer => sw.write_u8(4),
            SpectatorFrameData::SpectatingOther {user_id} => {sw.write_u8(5); sw.write(*user_id)},
            SpectatorFrameData::ReplayFrame {frame} => {sw.write_u8(6); sw.write(*frame)},
            &SpectatorFrameData::ScoreSync {score} => {sw.write_u8(7); sw.write(score.clone())},
            SpectatorFrameData::ChangingMap => sw.write_u8(9),

            SpectatorFrameData::PlayingResponse {user_id, beatmap_hash, mode, mods, current_time} => {write!(sw, 10u8, *user_id, beatmap_hash.clone(), *mode, mods.clone(), *current_time);},
        }
    }
}