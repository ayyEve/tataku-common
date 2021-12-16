use std::rc::Rc;

use crate::prelude::*;
use super::prelude::*;

#[derive(Clone)]
pub struct Image {
    pub pos: Vector2,
    pub size:Vector2,
    pub depth: f64,
    pub tex: Rc<Texture>,
    
    spawn_time:u64,
    scale: Vector2
}
impl Image {
    pub fn new(pos: Vector2, depth: f64, tex:Texture, size:Vector2) -> Image {
        // let scale = Vector2::new(tex.get_width() as f64 / size.x, tex.get_height() as f64 / size.y);
        let scale = Vector2::new(size.x / tex.get_width() as f64, size.y / tex.get_height() as f64);

        Image {
            pos,
            size,
            depth,
            tex: Rc::new(tex),
            spawn_time: 0,
            scale,
        }
    }
}
impl Renderable for Image {
    fn get_lifetime(&self) -> u64 {0}
    fn set_lifetime(&mut self, _lifetime:u64) {}
    fn get_spawn_time(&self) -> u64 {self.spawn_time}
    fn set_spawn_time(&mut self, time:u64) {self.spawn_time = time}
    fn get_depth(&self) -> f64 {self.depth}
    fn draw(&mut self, g: &mut GlGraphics, c: Context) {
        graphics::image(
            self.tex.as_ref(), 
            c.transform.trans(self.pos.x, self.pos.y).scale(self.scale.x, self.scale.y), 
            g);
    }
}