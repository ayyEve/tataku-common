mod benchmark_helper;
mod fps_display;
mod volume_control;
pub mod key_counter;
pub mod math;
pub mod curve;
pub mod skin_helper;
pub mod transform;
pub mod centered_text_helper;

pub use fps_display::*;
pub use volume_control::*;
pub use benchmark_helper::*;

pub mod io;
use crate::Vector2;

pub fn visibility_bg(pos:Vector2, size:Vector2, depth: f64) -> Box<crate::render::Rectangle> {
    let mut color = crate::render::Color::WHITE;
    color.a = 0.6;
    Box::new(crate::render::Rectangle::new(
        color,
        depth,
        pos,
        size,
        None
    ))
}