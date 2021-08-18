use piston::RenderArgs;

use taiko_rs_common::types::ScoreHit;
use crate::render::Renderable;
use super::modes::taiko::HitType;


// hitobject trait, implemented by anything that should be hit
pub trait HitObject: Send {
    fn note_type(&self) -> NoteType;
    fn is_kat(&self) -> bool {false}// needed for diff calc :/
    fn set_sv(&mut self, sv:f64);
    /// does this hit object play a finisher sound when hit?
    fn finisher_sound(&self) -> bool {false}

    /// time in ms of this hit object
    fn time(&self) -> u64;
    /// when should the hitobject be considered "finished", should the miss hitwindow be applied (specifically for notes)
    fn end_time(&self, hitwindow_miss:f64) -> u64;
    /// does this object count as a miss if it is not hit?
    fn causes_miss(&self) -> bool; //TODO: might change this to return an enum of "no", "yes". "yes_combo_only"
    
    fn get_points(&mut self, hit_type:HitType, time:f64, hit_windows:(f64,f64,f64)) -> ScoreHit; // if negative, counts as a miss
    fn check_finisher(&mut self, _hit_type:HitType, _time:f64) -> ScoreHit {ScoreHit::None}

    fn update(&mut self, beatmap_time: i64);
    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>>;

    /// set this object back to defaults
    fn reset(&mut self);

    fn x_at(&self, time:i64) -> f64;
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