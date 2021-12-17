use crate::prelude::*;
use super::prelude::*;

#[derive(Clone)]
pub struct Image {
    size: Vector2,
    pub depth: f64,
    pub tex: Arc<Texture>,

    
    // initial
    pub initial_pos: Vector2,
    pub initial_scale: Vector2,
    pub initial_rotation: f64,

    // current
    pub current_pos: Vector2,
    pub current_scale: Vector2,
    pub current_rotation: f64,
    
    spawn_time:u64,
}
impl Image {
    pub fn new(pos: Vector2, depth: f64, tex:Texture, size:Vector2) -> Image {
        // let scale = Vector2::new(tex.get_width() as f64 / size.x, tex.get_height() as f64 / size.y);
        let scale = Vector2::new(size.x / tex.get_width() as f64, size.y / tex.get_height() as f64);

        let initial_pos = pos;
        let initial_scale = scale;

        let current_pos = pos;
        let current_scale = scale;

        let initial_rotation = 0.0;
        let current_rotation = 0.0;

        Image {
            initial_pos,
            initial_scale,
            initial_rotation,

            current_pos,
            current_scale,
            current_rotation,

            size,
            depth,
            tex: Arc::new(tex),
            spawn_time: 0,
        }
    }

    pub fn size(&self) -> Vector2 {
        self.size * self.current_scale
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
        c
            .transform
            .trans(self.current_pos.x, self.current_pos.y)
            .scale(self.current_scale.x, self.current_scale.y)
            .rot_rad(self.current_rotation)
            , 
        g);
    }
}
impl Transformable for Image {
    fn apply_transform(&mut self, transform: &Transformation, val: TransformValueResult) {
        match transform.trans_type {
            TransformType::Position { .. } => {
                let val:Vector2 = val.into();
                // println!("val: {:?}", val);
                self.current_pos = self.initial_pos + val;
            },
            TransformType::Scale { .. } => {
                let val:f64 = val.into();
                self.current_scale = self.initial_scale + val;
            },
            TransformType::Rotation { .. } => {
                let val:f64 = val.into();
                self.current_rotation = self.current_rotation + val;
            }
            
            //TODO!
            TransformType::Transparency { .. } => {
                // let val:f64 = val.into();
                // self.current_color = self.current_color.alpha(val.clamp(0.0, 1.0) as f32);
            },

            // no color, ignore
            TransformType::Color { .. } => {},

            // this doesnt have a border, ignore
            TransformType::BorderTransparency { .. } => {},
            TransformType::BorderSize { .. } => {},
            TransformType::BorderColor { .. } => {},

            TransformType::None => {},
        }
    }
    
    fn visible(&self) -> bool {
        self.current_scale.x != 0.0 && self.current_scale.y != 0.0
    }

}