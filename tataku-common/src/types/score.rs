use crate::prelude::*;
use std::collections::HashMap;
use serde::{ Serialize, Deserialize };

// v2 added game speed as an f32
// v4 added custom judgments instead of sticking explicitly to osu judgment names
// v5 added time
// v6 changed mods to a hashset of mod ids
// v7 added performance value
// v9 moved the replay to the score object, removed the mods_string object, and changed mods to a Vec<ModDefinition>, also changed accuracy from f64 to f32
const CURRENT_VERSION:u16 = 9;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[derive(Reflect)]
pub struct Score {
    pub version: u16,
    pub username: String,
    pub beatmap_hash: Md5Hash,
    pub playmode: String,
    /// time in non-leap seconds since unix_epoch (UTC)
    pub time: u64,

    pub score: u64,
    pub combo: u16,
    pub max_combo: u16,

    pub judgments: HashMap<String, u16>,

    pub accuracy: f32,
    #[reflect(skip)]
    pub speed: GameSpeed,

    /// new mods format
    pub mods: Vec<ModDefinition>,

    /// how many performance points this is worth
    pub performance: f32,

    /// time diff for actual note hits. if the note wasnt hit, it wont be here
    /// (user_hit_time - correct_time)
    pub hit_timings: Vec<f32>,

    /// this was poorly documented when i initially added it to tataku.
    /// once i have the brain power to figure it out i'll update this doc
    pub stat_data: HashMap<String, Vec<f32>>,

    /// replay data for this score
    pub replay: Option<Replay>,
}
impl Score {
    pub fn new(beatmap_hash: Md5Hash, username: String, playmode: String) -> Score {
        Score {
            version: CURRENT_VERSION,
            username,
            beatmap_hash,
            playmode,
            time: 0,

            score: 0,
            combo: 0,
            max_combo: 0,
            performance: 0.0,

            judgments: HashMap::new(),
            accuracy: 0.0,
            speed: GameSpeed::default(),
            hit_timings: Vec::new(),
            mods: Vec::new(),
            stat_data: HashMap::new(),
            replay: None
        }
    }

    /// this is definitely going to break at some point, i need to figure out a better way to do this lol
    /// I was thinking of md5(format!("{time}{username}")), but this would break if time is 0 (default) :c
    pub fn hash(&self) -> String {
        let mut mods = format!("None");
        if self.version >= 3 {
            if self.mods.len() > 0 {
                let m = self.mods.iter().map(|m| m.name.clone()).collect::<Vec<String>>().join(",");
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                std::hash::Hash::hash(&m, &mut hasher);
                mods = format!("{:x}", std::hash::Hasher::finish(&hasher))
            }
        }

        // lol
        let x100 = self.judgments.get("x100")  .cloned().unwrap_or_default();
        let x300 = self.judgments.get("x300")  .cloned().unwrap_or_default();
        let xmiss = self.judgments.get("xmiss").cloned().unwrap_or_default();

        let judgments_str = self.judgment_string();
        let Score { beatmap_hash, score, combo, max_combo, playmode, .. } = &self;

        match self.version {
            // v3 still used manual judgments
            4.. => format!("{beatmap_hash}-{score},{combo},{max_combo},{judgments_str},{mods},{playmode}"),
            // v2 didnt have mods
            3 => format!("{beatmap_hash}-{score},{combo},{max_combo},{x100},{x300},{xmiss},{mods},{playmode}"),
            // v1 hash didnt have the playmode
            2 => format!("{beatmap_hash}-{score},{combo},{max_combo},{x100},{x300},{xmiss},{playmode}"),
            1 => format!("{beatmap_hash}-{score},{combo},{max_combo},{x100},{x300},{xmiss}"),
            0 => format!("what"),
        }
    }

    pub fn hit_error(&self) -> HitError {
        // from https://gist.github.com/peppy/3a11cb58c856b6af7c1916422f668899
        let mut total = 0.0;
        let mut _total = 0.0;
        let mut total_all = 0.0;
        let mut count = 0.0;
        let mut _count = 0.0;

        for &i in self.hit_timings.iter() {
            total_all += i;

            if i > 0.0 {
                total += i;
                count += 1.0;
            } else {
                _total += i;
                _count += 1.0;
            }
        }

        let mean = total_all / self.hit_timings.len() as f32;
        let mut variance = 0.0;
        for &i in self.hit_timings.iter() {
            variance += (i - mean).powi(2);
        }

        HitError {
            mean,
            early: _total / _count,
            late: total / count,
            deviance: (variance / self.hit_timings.len() as f32).sqrt()
        }
    }

    pub fn judgment_string(&self) -> String {
        let mut judgments = self.judgments.iter()
            .map(|(key, val)| format!("{key}:{val}"))
            .collect::<Vec<String>>();

        judgments.sort_unstable();
        judgments.join("|")
    }

    pub fn judgments_from_string(judgment_string: &String) -> HashMap<String, u16> {
        let mut judgments = HashMap::new();

        let entries = judgment_string.split("|");
        for entry in entries {
            let mut split = entry.split(":");
            if let Some((key, val)) = split.next().zip(split.next()) {
                let key = key.to_owned();
                let val = val.parse().unwrap();
                judgments.insert(key, val);
            }
        }

        judgments
    }

    pub fn get_judgment(&self, judgment: impl AsRef<str>) -> u16 {
        self.judgments.get(judgment.as_ref()).cloned().unwrap_or_default()
    }
}

// mods stuff
impl Score {
    /// get a sorted list of maps, separated by a comma and a space
    pub fn mods_string_sorted(&self) -> String {
        let mut mods = self.mods.iter().map(|m| m.name.clone()).collect::<Vec<_>>();
        mods.sort_unstable(); // unstable sort is fine because no two elements will ever be equal
        mods.join(", ")
    }
}

impl Serializable for Score {
    fn read(sr: &mut SerializationReader) -> SerializationResult<Self> {
        sr.push_parent("Score");

        let version = sr.read::<u16>("version")?;

        if version < 9 {
            return read_old_score(version, sr);
        }
        // macro_rules! version {
        //     ($v:expr, $def:expr) => {
        //         if version >= $v {
        //             sr.read()?
        //         } else {
        //             $def
        //         }
        //     };
        // }

        // everything here so far exists in v9
        let a = Ok(Score {
            version,
            username: sr.read("username")?,
            beatmap_hash: sr.read("beatmap_hash")?,
            playmode: sr.read("playmode")?,
            time: sr.read("time")?,

            score: sr.read("score")?,
            combo: sr.read("combo")?,
            max_combo: sr.read("max_combo")?,
            judgments: sr.read("judgments")?,
            accuracy: sr.read("accuracy")?,
            speed: GameSpeed::from_f32(sr.read("speed")?),

            mods: sr.read("mods")?,
            performance: sr.read("performance")?,

            // mods_string,
            stat_data: sr.read("stat_data")?,
            replay: sr.read("replay")?,

            hit_timings: Vec::new(),
        });

        sr.pop_parent();
        a
    }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write(&CURRENT_VERSION);
        sw.write(&self.username);
        sw.write(&self.beatmap_hash);
        sw.write(&self.playmode);
        sw.write(&self.time);

        sw.write(&self.score);
        sw.write(&self.combo);
        sw.write(&self.max_combo);

        sw.write(&self.judgments);

        sw.write(&self.accuracy);
        sw.write(&self.speed.as_f32());

        // sw.write(self.mods_string.clone());
        sw.write(&self.mods);
        sw.write(&self.performance);
        sw.write(&self.stat_data);

        sw.write(&self.replay);
    }
}



/// helper struct
#[derive(Copy, Clone, Debug)]
pub struct HitError {
    pub mean: f32,
    pub early: f32,
    pub late: f32,
    pub deviance: f32
}


/// legacy mod manager, only used to read old scores
#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct ModManager {
    pub speed: Option<u16>,

    pub easy: bool,
    pub hard_rock: bool,
    pub autoplay: bool,
    pub nofail: bool,
}

fn read_old_score(
    version: u16,
    sr: &mut SerializationReader,
) -> SerializationResult<Score> {
    macro_rules! version {
        ($v:expr, $a: expr, $def:expr) => {
            if version >= $v {
                sr.read($a)?
            } else {
                $def
            }
        };
    }

    sr.push_parent("Score (old)");

    let username = sr.read("username")?;
    let beatmap_hash = sr.read("beatmap_hash")?;
    let playmode = sr.read("playmode")?;
    let time = version!(5, "time", 0); // v5 added time

    let score = sr.read("score")?;
    let combo = sr.read("combo")?;
    let max_combo = sr.read("max_combo")?;

    // before version 4, judgments were stored manually
    let mut judgments = HashMap::new();
    if version < 4 {
        for i in [
            "x50",
            "x100",
            "x300",
            "xgeki",
            "xkatu",
            "xmiss"
        ] {
            let val:u16 = sr.read(i)?;
            judgments.insert(i.to_owned(), val);
        }
    } else {
        let count:usize = sr.read("judment count")?;
        for n in 0..count {
            let key = sr.read(format!("judgement key #{n}"))?;
            let val = sr.read(format!("judgment val #{n}"))?;
            judgments.insert(key, val);
        }
    }

    let accuracy:f64 = sr.read("accuracy")?;
    let speed = if version >= 2 {
        GameSpeed::from_f32(sr.read::<f32>("speed")?)
    } else {
        GameSpeed::default()
    };

    let mut mods2 = HashSet::new();
    let mut mods = Vec::new();

    // v 3-5 stored mods as a string
    match version {
        // v 0,1,2 did not have mods
        0..=2 => {}

        // v 3-5 stored mods as a string
        3..=5 => {
            // old mods
            let mods_string: Option<String> = sr.read("mods string")?;

            // parse here, save time later
            if let Some(str) = &mods_string {
                if let Ok(manager) = serde_json::from_str::<ModManager>(str) {
                    if manager.autoplay { mods2.insert("autoplay".to_owned()); }
                    if manager.nofail { mods2.insert("no_fail".to_owned()); }
                    if manager.hard_rock { mods2.insert("hard_rock".to_owned()); }
                    if manager.easy { mods2.insert("easy".to_owned()); }
                }
            }
        }

        // 6-8 stored mods as a hashset of mod ids
        6..=8 => {
            mods2 = sr.read("mods hashset")?;
        }

        // v9 started storing mod information in the score
        9.. => {
            mods = sr.read("mods")?;
        }

        // _ => unreachable!()
    }

    // if version >= 3 {
    //     if version <= 5 {
    //         let mut mods = HashSet::new();
    //         // old mods
    //         let mods_string: Option<String> = sr.read()?;
    //         // parse here, save time later
    //         if let Some(str) = &mods_string {
    //             if let Ok(manager) = serde_json::from_str::<ModManager>(str) {
    //                 if manager.autoplay { mods.insert("autoplay".to_owned()); }
    //                 if manager.nofail { mods.insert("no_fail".to_owned()); }
    //                 if manager.hard_rock { mods.insert("hard_rock".to_owned()); }
    //                 if manager.easy { mods.insert("easy".to_owned()); }
    //             }
    //         }
    //     } else {
    //         // new mods as of v6
    //         mods2 = sr.read()?;
    //     }
    // }


    if !mods2.is_empty() {
        mods = mods2.into_iter().map(|m| ModDefinition {
            name: m.clone(),
            short_name: format!("??"),
            display_name: m,
            adjusts_difficulty: false,
            score_multiplier: 1.0,
        }).collect();
    }

    let performance = version!(7, "performance", 0.0);
    let stat_data = version!(8, "stat_data", HashMap::new());

    let a = Ok(Score {
        version,
        username,
        beatmap_hash,
        playmode,
        time,

        score,
        combo,
        max_combo,
        judgments,
        accuracy: accuracy as f32,
        speed,
        performance,

        hit_timings: Vec::new(),

        mods,
        stat_data,
        replay: None,
    });

    sr.pop_parent();

    a
}



#[allow(unused)]
#[cfg(feature = "test")]
pub(super) mod tests {
    use crate::tests::*;
    use crate::prelude::*;
    use std::collections::HashMap;

    pub fn make_score<'a>(
        version: u16,
        replay: Option<RawOrOther<'a, Replay>>,
    ) -> Vec<u8> {
        let mut writer = VersionedWriter::new(version);

        writer.write(1, &version, "version");
        writer.write(1, &format!("username"), "username");
        writer.write(1, &Md5Hash::default(), "beatmap_hash");
        writer.write(1, &format!("osu"), "playmode");
        writer.write(5, &0u64, "time");
        writer.write(1, &5000i64, "score");
        writer.write(1, &50u16, "combo");
        writer.write(1, &50u16, "max_combo");

        // judgments
        let judgments = make_judgments();
        match version {
            0..=3 => {
                for i in [
                    "x50",
                    "x100",
                    "x300",
                    "xgeki",
                    "xkatu",
                    "xmiss"
                ] {
                    let val = judgments.get(i).copied().unwrap_or_default();
                    writer.write_ranged(0..=3, &val, i);
                }
            }

            4.. => {
                writer.write_ranged(4.., &judgments, "judgments");
            }
        }

        // accuracy
        let accuracy = 85.0f64;
        if version < 9 {
            writer.write_ranged(0..9, &accuracy, "accuracy");
        } else {
            writer.write_ranged(9.., &(accuracy as f32), "accuracy");
        }

        writer.write(2, &1.5f32, "speed");

        // mods
        let mods = make_mods(version);
        match (version, mods) {
            (0..=2, None) => {}

            // mods are stored as a string
            (3..=5, Some(mods)) => {
                let mut mod_manager = super::ModManager::default();
                let ModsDef::Old(mods) = &mods else { panic!("got new mods for old score version") };
                for (m, val) in [
                    ("autplay", &mut mod_manager.autoplay),
                    ("no_fail", &mut mod_manager.nofail),
                    ("hard_rock", &mut mod_manager.hard_rock),
                    ("easy", &mut mod_manager.easy),
                ] {
                    *val = mods.contains(m);
                }

                writer.write_ranged(3..=5,&serde_json::to_string(&mod_manager).unwrap(), "mods_string");
            }

            // mods are stored as hashset
            (6..=8, Some(mods)) => {
                let ModsDef::Old(mods) = &mods else { panic!("got new mods for old score version") };
                writer.write_ranged(6..=8, mods, "mods hashset");
            }

            // mods are stored as vec of ModDefinition
            (9.., Some(mods)) => {
                let ModsDef::New(mods) = &mods else { panic!("got old mods for new score version") };
                writer.write_ranged(9.., mods, "mods");
            }

            _=> unreachable!("bad mods")
        }

        writer.write(7, &40.0f32, "performance");
        writer.write(8, &make_stats(), "stats");

        if version >= 9 {
            use crate::types::replays;
            let replay = replay.unwrap_or_else(|| RawOrOther::Raw(replays::tests::make_replay(replays::CURRENT_VERSION)));
            writer.write_ranged(..9, &replay, "replay");
        }

        writer.data()
    }

    pub enum ModsDef {
        /// v4-v8
        Old(HashSet<String>),
        /// v9+
        New(Vec<ModDefinition>),
    }

    fn make_judgments() -> HashMap<String, u16> {
        [
            (format!("x300"), 40),
            (format!("xmiss"), 40),
        ].into_iter().collect()
    }


    fn make_mods(version: u16) -> Option<ModsDef> {
        match version {
            0..=2 => None,
            3..=8 => Some(ModsDef::Old([ "no_fail", "easy" ].into_iter().map(ToString::to_string).collect())),
            9.. => Some(ModsDef::New(vec![
                ModDefinition::new(
                    "no_fail",
                    "NF",
                    "No Fail",
                    false,
                    0.75
                ),
                ModDefinition::new(
                    "easy",
                    "EZ",
                    "Easy",
                    true,
                    0.75
                )
            ]))
        }
    }

    fn make_stats() -> HashMap<String, Vec<f32>> {
        [
            (format!("a"), vec![1.0, 2.0]),
            (format!("b"), vec![1.0, 2.0]),
        ].into_iter().collect()
    }

    #[test]
    fn try_read_scores() {
        for version in 1..CURRENT_VERSION {
            let score = make_score(version, None);
            let mut reader = SerializationReader::new(score);

            match Score::read(&mut reader) {
                Ok(_) => {}
                Err(e) => panic!("Error reading score v{version}: {e:?}"),
            }
        }
    }
}
