/// how many decimal places to "preserve"
const PRECISION:i32 = 2;
use serde::{ Serialize, Deserialize };

/// helper struct for speed multipliers
/// since we want them to be easily comparable (unlike f32s with floating point issues)
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[derive(Serialize, Deserialize)]
#[serde(from="u16", into="u16")]
pub struct GameSpeed(u16);
impl GameSpeed {
    #[inline]
    fn scale() -> f32 { (10f32).powi(PRECISION) }

    pub fn is_default(&self) -> bool { self == &Self::default()}

    pub fn from_u16(speed: u16) -> Self { Self(speed) }
    pub fn from_i32(speed: i32) -> Self { Self(speed as u16) }
    pub fn from_f32(speed: f32) -> Self { Self((speed * Self::scale()) as u16) }

    pub fn as_u16(&self) -> u16 { self.0 }
    pub fn as_i32(&self) -> i32 { self.0 as i32 }
    pub fn as_f32(&self) -> f32 { self.0 as f32 / Self::scale() }
}
// default speed is 1.0
impl Default for GameSpeed {
    fn default() -> Self { Self::from_f32(1.0) }
}

impl std::fmt::Debug for GameSpeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.as_f32(), self.0)
    }
}
impl std::fmt::Display for GameSpeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_f32().fmt(f)
    }
}

impl Into<u16> for GameSpeed {
    fn into(self) -> u16 { self.as_u16() }
}
impl From<u16> for GameSpeed {
    fn from(value: u16) -> Self { Self::from_u16(value) }
}

impl Into<i32> for GameSpeed {
    fn into(self) -> i32 { self.as_i32() }
}
impl From<i32> for GameSpeed {
    fn from(value: i32) -> Self { Self::from_i32(value) }
}

impl Into<f32> for GameSpeed {
    fn into(self) -> f32 { self.as_f32() }
}
impl From<f32> for GameSpeed {
    fn from(value: f32) -> Self { Self::from_f32(value) }
}

