use super::*;

use std::{fmt::Display, io::Error as IOError};
use serde_json::Error as JsonError;
use image::ImageError;

pub type TaikoResult<T> = Result<T, TaikoError>;

#[derive(Debug)]
#[allow(dead_code, unused)]
pub enum TaikoError {
    Beatmap(BeatmapError),
    IO(IOError),
    Serde(JsonError),

    Audio(AudioError),

    Image(ImageError)
}

impl Display for TaikoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            TaikoError::Beatmap(e) => write!(f, "{:?}", e),
            TaikoError::Serde(e) => write!(f, "{:?}", e),
            TaikoError::IO(e) => write!(f, "{}", e),
            TaikoError::Image(e) => write!(f, "{:?}", e),
            TaikoError::Audio(e) =>  write!(f, "{:?}", e),
        }
    }
}


impl From<JsonError> for TaikoError {
    fn from(e: JsonError) -> Self {TaikoError::Serde(e)}
}
impl From<IOError> for TaikoError {
    fn from(e: IOError) -> Self {TaikoError::IO(e)}
}
impl From<ImageError> for TaikoError {
    fn from(e: ImageError) -> Self {TaikoError::Image(e)}
}
impl From<AudioError> for TaikoError {
    fn from(e: AudioError) -> Self {TaikoError::Audio(e)}
}