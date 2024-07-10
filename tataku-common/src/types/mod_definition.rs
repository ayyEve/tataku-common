use crate::prelude::*;

// v1 added name, short_name, display_name, adjusts_difficulty, score_multiplier
const CURRENT_VERSION:u16 = 1;

/// a simple mod definition
#[derive(Clone, Debug, Serialize, Deserialize)]
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
        let version = sr.read_u16()?;

        macro_rules! version {
            ($version:expr, $default:expr) => {
                if version >= $version {
                    sr.read()?
                } else {
                    $default
                }
            };
        }

        Ok(Self {
            name: version!(1, String::new()),
            short_name: version!(1, String::new()),
            display_name: version!(1, String::new()),
            adjusts_difficulty: version!(1, false),
            score_multiplier: version!(1, 1.0),
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