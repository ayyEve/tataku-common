use crate::prelude::*;

/// used to determine the parent game for a map
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[derive(Serialize, Deserialize)]
#[derive(Reflect)]
#[reflect(from_string = "from_str")]
pub enum MapGame {
    #[default]
    Osu,
    Quaver,
    Other(String),
}

impl std::str::FromStr for MapGame {
    type Err = ReflectError<'static>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &*s.to_lowercase() {
            "osu" => Ok(MapGame::Osu),
            "quaver" => Ok(MapGame::Quaver),
            _ => Ok(MapGame::Other(s.to_owned())),
        }
    }
}

impl Serializable for MapGame {
    fn read(sr: &mut SerializationReader) -> SerializationResult<Self> where Self: Sized {
        let s:String = sr.read("MapGame")?;
        match &*s.to_lowercase() {
            "osu" => Ok(Self::Osu),
            "quaver" => Ok(Self::Quaver),
            _ => Ok(Self::Other(s)),
        }
    }

    fn write(&self, sw: &mut SerializationWriter) {
        match self {
            MapGame::Osu => sw.write(&"osu".to_owned()),
            MapGame::Quaver => sw.write(&"quaver".to_owned()),
            MapGame::Other(s) => sw.write(s),
        }
    }
}
