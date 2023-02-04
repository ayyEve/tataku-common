use crate::prelude::*;

pub type SpectatorFrame = (f32, SpectatorFrameData);
pub type SpectatorFrames = Vec<SpectatorFrame>;

#[derive(Clone, Debug)]
#[derive(PacketSerialization)]
#[Packet(type="u8")]
#[Packet(extra_logging)]
pub enum SpectatorFrameData {
    /// host started a new map
    /// NOTE: mods is a comma separated list of mod ids, ie "no_fail, autoplay"
    /// speed will need to be divided by 100
    #[Packet(id=0)]
    Play { beatmap_hash:String, mode: PlayMode, mods: String, speed: u16 },

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
    SpectatingOther { user_id:u32 },

    /// host pressed a game key
    #[Packet(id=5)]
    ReplayFrame { frame:ReplayFrame },

    /// clear up any score innaccuracies, or update new specs with this
    #[Packet(id=6)]
    ScoreSync { score:Score },

    /// host is changing the map
    #[Packet(id=7)]
    ChangingMap,

    /// response for current map info
    #[Packet(id=8)]
    /// NOTE: mods is a comma separated list of mod ids, ie "no_fail, autoplay"
    /// speed will need to be divided by 100
    PlayingResponse { user_id:u32, beatmap_hash:String, mode:PlayMode, mods:String, current_time:f32, speed: u16 },

    /// info about the map specified, might include a way to download it
    /// NOTE!! you should verify the download link instead of blindly trusting it
    #[Packet(id=9)]
    MapInfo { beatmap_hash: String, game: String, download_link: Option<String> },

    /// unknown packet
    #[Packet(id=255, default_variant)]
    Unknown,
}
