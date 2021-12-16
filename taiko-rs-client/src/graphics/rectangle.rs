use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct Rectangle {
    pub color: Color,
    pub depth: f64,
    pub pos: Vector2,
    pub size: Vector2,
    pub border: Option<Border>,

    spawn_time: u64,
    lifetime: u64
}
impl Rectangle {
    pub fn new(color: Color, depth: f64, pos: Vector2, size: Vector2, border: Option<Border>) -> Rectangle {
        Rectangle {
            color,
            depth,
            pos,
            size,
            border,

            spawn_time: 0,
            lifetime: 0
        }
    }
    
    /// helpful shortcut when you only want to measure text
    pub fn bounds_only(pos: Vector2, size: Vector2) -> Rectangle {
        Rectangle::new(Color::BLACK, 0.0, pos, size, None)
    }

    /// check if this rectangle contains a point
    pub fn contains(&self, p:Vector2) -> bool {
        p.x > self.pos.x && p.x < self.pos.x + self.size.x && p.y > self.pos.y && p.y < self.pos.y + self.size.y
    }
}
impl Renderable for Rectangle {
    fn get_depth(&self) -> f64 {self.depth}
    fn set_lifetime(&mut self, lifetime:u64) {self.lifetime = lifetime}
    fn get_lifetime(&self) -> u64 {self.lifetime}
    fn set_spawn_time(&mut self, time:u64) {self.spawn_time = time}
    fn get_spawn_time(&self) -> u64 {self.spawn_time}

    fn draw(&mut self, g: &mut GlGraphics, c: Context) {
        let mut r = graphics::Rectangle::new(self.color.into());
        if let Some(b) = self.border {r.border = Some(b.into())}
        r.draw([
            self.pos.x, self.pos.y, 
            self.size.x, self.size.y
        ], &DrawState::default(),c.transform, g);
    }
}
