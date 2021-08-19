use piston::RenderArgs;

use taiko_rs_common::types::ScoreHit;
use crate::render::Renderable;
use super::modes::taiko::HitType;


// hitobject trait, implemented by anything that should be hit
pub trait HitObject: Send {
    fn note_type(&self) -> NoteType;

    /// time in ms of this hit object
    fn time(&self) -> u64;
    /// when should the hitobject be considered "finished", should the miss hitwindow be applied (specifically for notes)
    fn end_time(&self, hitwindow_miss:f64) -> u64;

    fn update(&mut self, beatmap_time: i64);
    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>>;

    /// set this object back to defaults
    fn reset(&mut self);
}

/// only used for diff calc
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NoteType {
    Note,
    Slider,
    Spinner,
    /// mania only
    Hold
}