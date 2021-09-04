use super::*;

use std::io::Error as IOError;


pub enum TaikoError {
    Beatmap(BeatmapError),
    IO(IOError)
}