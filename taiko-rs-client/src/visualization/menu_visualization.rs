
use std::time::Instant;

use ayyeve_piston_ui::render::{Color, Line, Vector2, Renderable};

use super::Visualization;
use crate::{WINDOW_SIZE, game::Audio};


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
    fn data(&mut self) -> &mut Vec<f32> {&mut self.data}
    fn timer(&mut self) -> &mut Instant {&mut self.timer} 

    fn draw(&mut self, _args:piston::RenderArgs, _pos_offset:Vector2, depth:f64, list:&mut Vec<Box<dyn Renderable>>) {
        self.update_data();

        // let mut graph = Graph::new(
        //     Vector2::zero(), 
        //     Vector2::new(500.0, 500.0),
        //     self.data.clone(),
        //     -10.0, 10.0
        // );
        // list.extend(graph.draw(args, Vector2::new(0.0, 0.0), depth));


        let mid = WINDOW_SIZE / 2.0;
        let inner_radius = 100.0;

        let a = self.data.len() as f64 / 360.0;
        for i in 0..self.data.len() {
            let theta = (a * i as f64).to_radians();
            let cos = theta.cos();
            let sin = theta.sin();
            let p1 = mid + Vector2::new(
                 cos * inner_radius,
                 sin * inner_radius
            );

            let l = inner_radius + self.data[i] as f64 * 5.0;
            let p2 = mid + Vector2::new(
                cos * l,
                sin * l
            );

            list.push(Box::new(Line::new(
                p1,
                p2,
                2.0,
                depth,
                Color::BLUE
            )));
        }
    }
}


