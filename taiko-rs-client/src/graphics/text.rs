use crate::prelude::*;

#[derive(Clone)]
pub struct Text {
    pub color: Color,
    pub depth: f64,
    pub pos: Vector2,
    pub font_size: u32,
    pub text: String,
    pub font: Font,

    lifetime:u64,
    spawn_time:u64
}
impl Text {
    pub fn new(color:Color, depth:f64, pos: Vector2, font_size: u32, text: String, font: Font) -> Text {
        Text {
            color,
            depth,
            pos,
            font_size,
            text,
            font,
            lifetime: 0,
            spawn_time: 0
        }
    }
    pub fn measure_text(&self) -> Vector2 {
        let mut text_size = Vector2::zero();
        let mut font = self.font.lock();

        // let block_char = '█';
        // let character = font.character(self.font_size, block_char).unwrap();

        for _ch in self.text.chars() {
            let character = font.character(self.font_size, _ch).unwrap();
            text_size.x += character.advance_width();
            // text_size.y = text_size.y.max(character.offset[1]); //character.advance_height();
        }
        
        text_size
    }
    pub fn center_text(&mut self, rect:Rectangle) {
        let text_size = self.measure_text();
        self.pos = rect.pos + (rect.size - text_size)/2.0 + Vector2::new(0.0, text_size.y);
    }
}
impl Renderable for Text {
    fn get_depth(&self) -> f64 {self.depth}
    fn set_lifetime(&mut self, lifetime:u64) {self.lifetime = lifetime}
    fn get_lifetime(&self) -> u64 {self.lifetime}
    fn set_spawn_time(&mut self, time:u64) {self.spawn_time = time}
    fn get_spawn_time(&self) -> u64 {self.spawn_time}

    fn draw(&mut self, g: &mut GlGraphics, c: Context) {
        graphics::text(
            self.color.into(),
            self.font_size,
            self.text.as_str(),
            &mut *self.font.lock(),
            c.transform.trans(self.pos.x, self.pos.y),
            g
        ).unwrap();
    }
}