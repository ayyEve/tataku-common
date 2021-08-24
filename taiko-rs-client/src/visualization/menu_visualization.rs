
use std::time::Instant;

use ayyeve_piston_ui::render::{Color, Line, Vector2, Renderable};

use super::Visualization;
use crate::WINDOW_SIZE;

const CUTOFF:f64 = 1.0;
const COLORS:[Color; 3] = [
    Color::RED,
    Color::BLUE,
    Color::GREEN,
];


pub struct MenuVisualization {
    data: Vec<f32>,
    timer: Instant
}
impl MenuVisualization {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            timer: Instant::now()
        }
    }

}

impl Visualization for MenuVisualization {
    fn lerp_factor(&self) -> f32 {15.0}
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

        let inner_radius = 100.0;

        let a = 360.0 / self.data.len() as f64;
        let n = (2.0 * std::f64::consts::PI * inner_radius) / self.data.len() as f64/2.0;

        for i in 0..self.data.len() {
            let theta = (a * i as f64).to_radians();
            let cos = theta.cos();
            let sin = theta.sin();
            let p1 = pos + Vector2::new(
                cos * inner_radius,
                sin * inner_radius
            );

            const MULT:f64 = 1.0; // 5.0
            let l = inner_radius + self.data[i].abs() as f64 * MULT;
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


