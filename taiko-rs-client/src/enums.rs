

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum Playmode {
    Standard,
    Taiko,
    Catch,
    Mania
}
impl Into<Playmode> for u8 {
    fn into(self) -> Playmode {
        match self {
            0 => Playmode::Standard,
            1 => Playmode::Taiko,
            2 => Playmode::Catch,
            3 => Playmode::Mania,
            _ => Playmode::Standard
        }
    }
}

