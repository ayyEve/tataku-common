use crate::prelude::*;

mod benchmark_helper;
mod fps_display;
mod volume_control;
pub mod io;
pub mod math;
pub mod curve;
pub mod transform;
pub mod key_counter;
pub mod skin_helper;
pub mod centered_text_helper;

pub use fps_display::*;
pub use volume_control::*;
pub use benchmark_helper::*;


pub fn visibility_bg(pos:Vector2, size:Vector2, depth: f64) -> Box<Rectangle> {
    let mut color = Color::WHITE;
    color.a = 0.6;
    Box::new(Rectangle::new(
        color,
        depth,
        pos,
        size,
        None
    ))
}