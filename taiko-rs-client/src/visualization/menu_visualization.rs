
use std::time::Instant;

use ayyeve_piston_ui::render::{Color, Image, Line, Renderable, Vector2};
use opengl_graphics::{Texture, TextureSettings};

use super::Visualization;

const CUTOFF:f64 = 1.0;
// const COLORS:[Color; 3] = [
//     Color::RED,
//     Color::BLUE,
//     Color::GREEN,
// ];
const INNER_RADIUS:f64 = 100.0;


pub struct MenuVisualization {
    data: Vec<f32>,
    timer: Instant,

    cookie: Image
}
impl MenuVisualization {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            timer: Instant::now(),
            cookie: Image::new(Vector2::zero(), 0.0, Texture::empty(&TextureSettings::new()).unwrap(), Vector2::new(INNER_RADIUS, INNER_RADIUS))
        }
    }
}

impl Visualization for MenuVisualization {
    fn lerp_factor(&self) -> f32 {10.0} // 15
    fn data(&mut self) -> &mut Vec<f32> {&mut self.data}
    fn timer(&mut self) -> &mut Instant {&mut self.timer}


    fn draw(&mut self, _args:piston::RenderArgs, pos:Vector2, depth:f64, list:&mut Vec<Box<dyn Renderable>>) {
        self.update_data();

        // let mut graph = Graph::new(
        //     Vector2::zero(), 
        //     Vector2::new(500.0, 500.0),
        //     self.data.clone(),
        //     -10.0, 10.0
        // );
        // list.extend(graph.draw(args, Vector2::new(0.0, 0.0), depth));


        let a = 360.0 / self.data.len() as f64;
        let n = (2.0 * std::f64::consts::PI * INNER_RADIUS) / self.data.len() as f64/2.0;

        for i in 0..self.data.len() {
            let theta = (a * i as f64).to_radians();
            let cos = theta.cos();
            let sin = theta.sin();
            let p1 = pos + Vector2::new(
                cos * INNER_RADIUS,
                sin * INNER_RADIUS
            );


            const MULT:f64 = 5.0;
            let factor = (i as f64 + 2.0).log(10.0);
            let l = INNER_RADIUS + self.data[i].abs() as f64 * factor * MULT;

            if l < CUTOFF {continue}
            let p2 = pos + Vector2::new(
                cos * l,
                sin * l
            );

            list.push(Box::new(Line::new(
                p1,
                p2,
                n,
                depth,
                // COLORS[i % COLORS.len()]
                Color::BLUE
            )));
        }
    }

    fn reset(&mut self) {
        self.data = Vec::new();
        self.timer = Instant::now();
    }
}
