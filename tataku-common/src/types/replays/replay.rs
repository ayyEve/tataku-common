use crate::macros::*;
use crate::reflection::*;
use crate::serialization::*;
use std::collections::HashMap;

use crate::types::{
    Score,
    replays::ReplayFrame
};

// all versions have the frame data and version number
// v1 had a playmode entry, which was only relevent to taiko. removed in v2
// v2+ has score data included in the replay data
// v4+ has gamemode data (technically in v3 but i messed up whoops)
// v5+ has map time offset
// v6 had breaking changes since we moved the replay to the score (whereas before the score used to be in the replay)
pub(crate) const CURRENT_VERSION:u16 = 6;

#[derive(Clone, Debug, Default)]
#[derive(Serialize, Deserialize)]
#[derive(Reflect)]
pub struct Replay {
    // removed in v6
    // score: Option<Score>

    /// any extra gameplay variables which are helpful to know
    pub gamemode_data: HashMap<String, String>,

    /// time offset 
    pub offset: f32,

    /// (time, key)
    pub frames: Vec<ReplayFrame>, 
}
impl Replay {
    pub fn new() -> Self {
        Self {
            gamemode_data: HashMap::new(),
            offset: 0.0,
            frames: Vec::new(),
        }
    }

    /// helper function for reading replays.
    /// old replays had a different format so this allows for backwards compatability
    pub fn try_read_replay(sr: &mut SerializationReader) -> Result<Score, ReplayLoadError> {
        sr.push_parent("Replay (as Score)");
        // all versions wrote the version number
        let version = sr.read::<u16>("version")?;

        macro_rules! version {
            ($v:expr, $a: expr) => {
                if version >= $v {
                    sr.read($a)?
                } else {
                    Default::default()
                }
            };
            ($v:expr, $a: expr, $def:expr) => {
                if version >= $v {
                    sr.read($a)?
                } else {
                    $def
                }
            };
        }

        // if version is less than 6
        if version < 6 {
            if version == 1 { return Err(ReplayLoadError::TooOld) }

            let score: Option<Score> = sr.read("score")?;
            let Some(mut score) = score else {return Err(ReplayLoadError::NoScore)};


            score.replay = Some(Replay {
                gamemode_data: version!(4, "gamemode_data"),
                offset: version!(5, "offset"),
                frames: sr.read("frames")?,
            });

            sr.pop_parent();

            Ok(score)
        } else {
            // newer version will just be the score directly

            // unread the version
            sr.unread(std::mem::size_of::<u16>());

            // reread everything as a score
            let score:Score = sr.read("score")?;

            sr.pop_parent();

            Ok(score)
        }
    }
}
impl Serializable for Replay {
    fn read(sr: &mut SerializationReader) -> SerializationResult<Self> {
        let mut r = Replay::new();
        sr.push_parent("Replay");
        
        // all versions wrote the version number
        let version = sr.read::<u16>("version")?;

        if version < 6 {
            if version == 1 { let _playstyle = sr.read::<u8>("playstyle")?; }

            if version > 1 {
                let _score: Option<Score> = sr.read("score (unused)")?;
            }
                
            return Ok(Replay {
                gamemode_data: if version >= 4 { sr.read("gamemode_data")? } else { Default::default() },
                offset: if version >= 5 { sr.read("offset")? } else { Default::default() },
                frames: sr.read("frames")?,
            });
        }

        r.gamemode_data = sr.read("gamemode_data")?;
        r.offset = sr.read("offset")?;
        r.frames = sr.read("frames")?;
        
        Ok(r)
    }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write(&CURRENT_VERSION); // all versions
        // sw.write(self.playstyle as u8); // removed in v2
        // sw.write(&self.score_data); // added in v2, removed in v6
        sw.write(&self.gamemode_data); // added in v3, fixed in v4
        sw.write(&self.offset); // added in v5

        // println!("writing {} replay frames", self.frames.len());
        sw.write(&self.frames); // all versions
    }
}

#[derive(Debug)]
pub enum ReplayLoadError {
    TooOld,
    NoScore,
    SerializationError(SerializationError)
}
impl From<SerializationError> for ReplayLoadError {
    fn from(value: SerializationError) -> Self {
        Self::SerializationError(value)
    }
}

#[cfg(feature = "test")]
pub(crate) mod tests {
    use crate::tests::*;
    use crate::prelude::*;
    use std::collections::HashMap;

    fn make_replay_inner<'a>(
        version: u16,
        playstyle: Option<u8>,
        score: Option<impl Into<RawOrOther<'a, Score>>>,
        gamemode_data: Option<&HashMap<String, String>>,
        offset: Option<f32>,
        frames: &Vec<ReplayFrame>,
    ) -> Vec<u8> {
        let mut writer = SerializationWriter::new();
        writer.write::<u16>(&version); // write version

        // write playstyle (was only in v1)
        if version == 1 {
            let playstyle = playstyle.expect("no playstyle provided for v1 replay");
            writer.write::<u8>(&playstyle); 
        }

        if (2..=5).contains(&version) {
            let score = score.expect("no score provided for replay version 2..6");
            let score: RawOrOther<'a, Score> = score.into();
            let score = Some(score);
            writer.write(&score);
        }

        // write gamemode data
        if version >= 4 {
            let gamemode_data = gamemode_data.expect("no gamemode_data provided for replay version >= 4");
            writer.write(gamemode_data); 
        }

        // write offset
        if version >= 5 {
            let offset = offset.expect("no offset provided for replay version >= 5");
            writer.write::<f32>(&offset); 
        }

        // write frames
        writer.write(frames);

        // return data created
        writer.data()
    }

    pub fn make_replay(version: u16) -> Vec<u8> {
        use crate::types::score;
        /// what score version was the last to not have replays?
        const REPLAY_SCORE_VERSION_WITHOUT_REPLAY: u16 = 8;

        make_replay_inner(
            version, 
            (version == 1).then_some(2),
            (2..=5).contains(&version).then(|| score::tests::make_score(REPLAY_SCORE_VERSION_WITHOUT_REPLAY, None)),
            (version >= 4).then_some(&make_gamemode_data()),
            (version >= 5).then_some(5.0), 
            &make_frames(),
        )
    }

    fn make_frames() -> Vec<ReplayFrame> {
        vec![
            ReplayFrame::new(0.0, ReplayAction::Press(KeyPress::Left)),
            ReplayFrame::new(10.0, ReplayAction::Release(KeyPress::Left)),

            ReplayFrame::new(20.0, ReplayAction::Press(KeyPress::Right)),
            ReplayFrame::new(30.0, ReplayAction::Release(KeyPress::Right)),
        ]
    }

    fn make_gamemode_data() -> HashMap<String, String> {
        HashMap::new()
    }

    #[test]
    fn try_read_replays() {
        for version in 1..CURRENT_VERSION {
            println!("making replay v{version}");
            let replay = make_replay(version);

            println!("reading replay v{version}");
            let mut reader = SerializationReader::new(replay);

            if version < 6 {
                match Replay::try_read_replay(&mut reader) {
                    Ok(_) if version == 1 => panic!("somehow read v1 replay ???"),
                    Ok(_) => {}
                    // this is expected for version 1
                    Err(ReplayLoadError::TooOld) if version == 1 => {}

                    Err(ReplayLoadError::SerializationError(e)) => panic!("Failed to deserialize v{version} replay: {e}"),
                    Err(e) => panic!("Failed to deserialize v{version} replay: {e:?}"),
                }
            } else {
                match Replay::read(&mut reader) {
                    Ok(_) => {}
                    Err(e) => panic!("Failed to deserialize v{version} replay: {e:?}")
                }
            }

        }
    }
}
