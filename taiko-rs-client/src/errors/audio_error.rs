
use bass::prelude::BassError;
use symphonia::core::errors::Error as SymphoniaError;

#[derive(Debug)]
pub enum AudioError {
    SymphoniaError(SymphoniaError),
    BassError(BassError),
    
    DifferentSong,
}


impl From<SymphoniaError> for AudioError {
    fn from(e: SymphoniaError) -> Self {AudioError::SymphoniaError(e)}
}
impl From<BassError> for AudioError {
    fn from(e: BassError) -> Self {AudioError::BassError(e)}
}