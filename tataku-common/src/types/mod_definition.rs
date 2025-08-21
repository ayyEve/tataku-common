use crate::prelude::*;

// v1 added name, short_name, display_name, adjusts_difficulty, score_multiplier
const CURRENT_VERSION:u16 = 1;

/// a simple mod definition
#[derive(Reflect)]
#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct ModDefinition {
    /// mod identifier, used in the mods hashmap
    pub name: String,

    /// short (usually 2 letter) name for the mod (ie HR, EZ)
    pub short_name: String,

    /// actual display name for the mod
    pub display_name: String,

    /// does this mod adjust the difficulty rating? used for diff calc
    pub adjusts_difficulty: bool,

    /// how much does this mod adjust the score multiplier?
    pub score_multiplier: f32,
}
impl ModDefinition {
    pub fn new(
        name: impl ToString,
        short_name: impl ToString,
        display_name: impl ToString,
        adjusts_difficulty: bool,
        score_multiplier: f32,
    ) -> Self {
        Self {
            name: name.to_string(),
            short_name: short_name.to_string(),
            display_name: display_name.to_string(),
            adjusts_difficulty,
            score_multiplier,
        }
    }
}

impl AsRef<str> for ModDefinition {
    fn as_ref(&self) -> &str {
        &self.name
    }
}

impl PartialEq for ModDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for ModDefinition {}
impl std::hash::Hash for ModDefinition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl std::cmp::PartialOrd for ModDefinition {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl std::cmp::Ord for ModDefinition {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl Serializable for ModDefinition {
    fn read(sr: &mut SerializationReader) -> SerializationResult<Self> {
        sr.push_parent("ModDefinition");

        let version = sr.read::<u16>("version")?;

        macro_rules! version {
            ($version:expr, $a:expr, $default:expr) => {
                if version >= $version {
                    sr.read($a)?
                } else {
                    $default
                }
            };
        }

        Ok(Self {
            name: version!(1, "name", String::new()),
            short_name: version!(1, "short_name", String::new()),
            display_name: version!(1, "display_name", String::new()),
            adjusts_difficulty: version!(1, "adjusts_difficulty", false),
            score_multiplier: version!(1, "score_multiplier", 1.0),
        })
    }

    fn write(&self, sw: &mut SerializationWriter) {
        sw.write(&CURRENT_VERSION);
        sw.write(&self.name);
        sw.write(&self.short_name);
        sw.write(&self.display_name);
        sw.write(&self.adjusts_difficulty);
        sw.write(&self.score_multiplier);
    }
}
