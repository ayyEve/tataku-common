use crate::prelude::*;

/// This is a helper struct to help reduce memory usage, and improve hash comparison times (O(1) vs O(n))
/// 
/// This is primarily used for Beatmap hashes, but can be use for any md5 hash
/// 
/// Note that this item is serialized and deserialized as a string, in the usual md5 hash format
#[derive(Copy, Clone, Eq, Default, Debug, PartialEq, Hash, Serialize, Deserialize)]
#[serde(try_from="String", into="String")]
pub struct Md5Hash(u128);

impl TryFrom<&String> for Md5Hash {
    type Error = std::num::ParseIntError;

    fn try_from(s:&String) -> Result<Self, Self::Error> {
        Ok(Self(u128::from_str_radix(s, 16)?))
    }
}
impl TryFrom<String> for Md5Hash {
    type Error = std::num::ParseIntError;

    fn try_from(s:String) -> Result<Self, Self::Error> {
        Ok(Self(u128::from_str_radix(&s, 16)?))
    }
}

impl Into<String> for Md5Hash {
    fn into(self) -> String {
        self.to_string()
    }
}


impl AsRef<u128> for Md5Hash {
    fn as_ref(&self) -> &u128 {
        &self.0
    }
}

impl From<u128> for Md5Hash {
    fn from(value: u128) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for Md5Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl Serializable for Md5Hash {
    fn read(sr: &mut crate::SerializationReader) -> crate::SerializationResult<Self> where Self: Sized {
        let s = sr.read::<String>("md5")?;
        let r = Ok(s.try_into()?);
        r
    }

    fn write(&self, sw:&mut crate::SerializationWriter) {
        let s = self.to_string();
        sw.write(&s);
    }
}

#[test]
fn beatmap_hash_test() {
    let hash = "8bfe194c8bd641937d61e3995872fdba".to_string();
    let hash2:Md5Hash = (&hash).try_into().unwrap();
    let hash3:u128 = 0x8bfe194c8bd641937d61e3995872fdba;

    assert_eq!(hash2.to_string(), hash);
    assert_eq!(hash2.as_ref(), &hash3);
}