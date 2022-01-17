use crate::prelude::*;
use crate::{serialization::Serializable, ReplayFrame, PlayMode};
use super::Score;

pub type SpectatorFrame = (f32, SpectatorFrameData);
pub type SpectatorFrames = Vec<SpectatorFrame>;

#[derive(Clone, Debug)]
#[derive(PacketSerialization)]
#[Packet(type="u8")]
pub enum SpectatorFrameData {
    /// host started a new map
    #[Packet(id=0)]
    Play {beatmap_hash:String, mode: PlayMode, mods: String},

    /// host paused current map
    #[Packet(id=1)]
    Pause,

    // host unpaused the current map
    #[Packet(id=2)]
    UnPause,

    /// indicates the last time in a packet, so we know where we have data up to.
    /// should probably be renamed but whatever
    #[Packet(id=3)]
    Buffer,

    /// host started spectating someone else. deal with this later
    #[Packet(id=4)]
    SpectatingOther {user_id:u32},

    /// host pressed a game key
    #[Packet(id=5)]
    ReplayFrame {frame:ReplayFrame},

    /// clear up any score innaccuracies, or update new specs with this
    #[Packet(id=6)]
    ScoreSync {score:Score},

    /// host is changing the map
    #[Packet(id=7)]
    ChangingMap,

    /// response for current map info
    #[Packet(id=8)]
    PlayingResponse {user_id:u32, beatmap_hash:String, mode:PlayMode, mods:String, current_time:f32},

    /// unknown packet
    #[Packet(id=255, default_variant)]
    Unknown,
}
