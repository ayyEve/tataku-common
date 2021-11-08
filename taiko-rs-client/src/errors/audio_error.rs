
use symphonia::core::errors::Error as SymphoniaError;

#[derive(Debug)]
pub enum AudioError {
    SymphoniaError(SymphoniaError),

}


impl From<SymphoniaError> for AudioError {
    fn from(e: SymphoniaError) -> Self {AudioError::SymphoniaError(e)}
}