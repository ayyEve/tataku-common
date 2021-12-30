use crate::{serialization::Serializable, ReplayFrame, PlayMode};
use super::Score;

pub type SpectatorFrame = (u32, SpectatorFrameData);
pub type SpectatorFrames = Vec<SpectatorFrame>;

#[derive(Clone, Debug)]
pub enum SpectatorFrameData {
    /// host started a new map
    Play {beatmap_hash:String, mode: PlayMode},
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
    // host pressed a game key
    ReplayFrame {frame:ReplayFrame},

    // clear up any score innaccuracies, or update new specs with this
    ScoreSync {score:Score}
}
impl Serializable for SpectatorFrameData {
    fn read(sr:&mut crate::serialization::SerializationReader) -> Self {
        match sr.read_u8() {
            // play
            0 => SpectatorFrameData::Play {beatmap_hash: sr.read_string(), mode: sr.read()},
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

            // unknown
            n => panic!("unknown replay packet num: {}", n), // unknown
        }
    }

    fn write(&self, sw:&mut crate::serialization::SerializationWriter) {
        match &self {
            SpectatorFrameData::Play {beatmap_hash, mode} => {sw.write_u8(0); sw.write(beatmap_hash.clone()); sw.write(*mode);},
            SpectatorFrameData::Pause => sw.write_u8(1),
            SpectatorFrameData::UnPause => sw.write_u8(2),
            SpectatorFrameData::Stop => sw.write_u8(3),
            SpectatorFrameData::Buffer => sw.write_u8(4),
            SpectatorFrameData::SpectatingOther {user_id} => {sw.write_u8(5); sw.write(*user_id)},
            SpectatorFrameData::ReplayFrame {frame} => {sw.write_u8(6); sw.write(*frame)},
            &SpectatorFrameData::ScoreSync {score} => {sw.write_u8(7); sw.write(score.clone())}
        }
    }
}