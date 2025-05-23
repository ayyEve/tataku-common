use crate::prelude::*;

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Default)]
#[derive(Reflect, PacketSerialization)]
#[packet(type="u8")]
pub enum SpectatorAction {
    /// host started a new map
    /// NOTE: mods is a comma separated list of mod ids, ie "no_fail, autoplay"
    /// speed will need to be divided by 100
    #[packet(id=0)]
    Play {
        beatmap_hash: Md5Hash,
        mode: String,
        mods: Vec<ModDefinition>,
        speed: u16,

        map_game: MapGame,
        map_link: Option<String>
    },

    /// host paused current map
    #[packet(id=1)]
    Pause,

    // host unpaused the current map
    #[packet(id=2)]
    UnPause,

    /// indicates the last time in a packet, so we know where we have data up to.
    /// should probably be renamed but whatever
    #[packet(id=3)]
    Buffer,

    /// host started spectating someone else. deal with this later
    #[packet(id=4)]
    SpectatingOther { user_id: u32 },

    /// host pressed a game key
    #[packet(id=5)]
    ReplayAction { action: ReplayAction },

    /// clear up any score innaccuracies, or update new specs with this
    #[packet(id=6)]
    ScoreSync { score: Score },

    /// host is changing the map
    #[packet(id=7)]
    ChangingMap,

    /// the time has jumped
    ///
    /// usually used when the player joins spec mid-map
    #[packet(id=8)]
    TimeJump { time: f32 },

    // /// response for current map info
    // #[packet(id=8)]
    // /// NOTE: mods is a comma separated list of mod ids, ie "no_fail, autoplay"
    // /// speed will need to be divided by 100
    // PlayingResponse { user_id: u32, beatmap_hash: String, mode: String, mods: String, current_time: f32, speed: u16 },

    // /// info about the map specified, might include a way to download it
    // /// NOTE!! you should verify the download link instead of blindly trusting it
    // #[packet(id=9)]
    // MapInfo { beatmap_hash: String, game: String, download_link: Option<String> },

    /// unknown packet
    #[default]
    #[packet(id=255)]
    Unknown,
}
