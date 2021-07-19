use std::time::Instant;
use crate::{game::Vector2, render::{Color, Text}};

/// fps display helper, cleans up some of the code in game
pub struct FpsDisplay {
    name:String,
    pos:Vector2,
    count: u32,
    timer: Instant,
    last: f32,
}

impl FpsDisplay {
    /// name is what to display in text, count is which fps counter is this (only affects position)
    pub fn new(name:&str, count:u8) -> Self {
        Self {
            count: 0,
            last: 0.0,
            timer: Instant::now(),
            name: name.to_owned(),
            pos: Vector2::new(0.0, 10.0 + 20.0 * count as f64)
        }
    }

    pub fn increment(&mut self) {self.count += 1}
    pub fn draw(&mut self) -> Text {
        let font = crate::game::get_font("main");

        let fps_elapsed = self.timer.elapsed().as_micros() as f64 / 1000.0;
        if fps_elapsed >= 100.0 {
            self.last = (self.count as f64 / fps_elapsed * 1000.0) as f32;
            self.timer = Instant::now();
            self.count = 0;
        }

        Text::new(
            Color::BLACK,
            -99_999_999.0, // should be on top of everything
            self.pos,
            12,
            format!("{:.2}{}", self.last, self.name),
            font.clone()
        )
    }
}