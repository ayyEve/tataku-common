use super::*;

use std::{fmt::Display, io::Error as IOError};

use serde_json::Error as JsonError;

#[derive(Debug)]
pub enum TaikoError {
    Beatmap(BeatmapError),
    IO(IOError),
    Serde(JsonError),
}

impl Display for TaikoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            TaikoError::Beatmap(e) => write!(f, "{:?}", e),
            TaikoError::Serde(e) => write!(f, "{:?}", e),
            TaikoError::IO(e) => write!(f, "{}", e),
        }
    }
}


impl From<JsonError> for TaikoError {
    fn from(e: JsonError) -> Self {TaikoError::Serde(e)}
}
impl From<IOError> for TaikoError {
    fn from(e: IOError) -> Self {TaikoError::IO(e)}
}