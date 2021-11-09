use std::time::Instant;
use opengl_graphics::{Texture, TextureSettings};
use ayyeve_piston_ui::render::{Color, Image, Line, Renderable, Vector2};

use super::{FFTEntry, Visualization};

const CUTOFF:f32 = 0.1;
// const COLORS:[Color; 3] = [
//     Color::RED,
//     Color::BLUE,
//     Color::GREEN,
// ];
const INNER_RADIUS:f64 = 100.0;


pub struct MenuVisualization {
    data: Vec<FFTEntry>,
    timer: Instant,

    bar_height: f64,
    rotation: f64,

    cookie: Image
}
impl MenuVisualization {
    pub fn new() -> Self {
        Self {
            rotation: 0.0,
            data: Vec::new(),
            timer: Instant::now(),
            cookie: Image::new(Vector2::zero(), 0.0, Texture::empty(&TextureSettings::new()).unwrap(), Vector2::new(INNER_RADIUS, INNER_RADIUS)),

            bar_height: 1.0 //(Settings::get_mut().window_size[1] - INNER_RADIUS) / 128.0
        }
    }
}

impl Visualization for MenuVisualization {
    fn lerp_factor(&self) -> f32 {10.0} // 15
    fn data(&mut self) -> &mut Vec<FFTEntry> {&mut self.data}
    fn timer(&mut self) -> &mut Instant {&mut self.timer}

    fn draw(&mut self, _args:piston::RenderArgs, pos:Vector2, depth:f64, list:&mut Vec<Box<dyn Renderable>>) {
        let since_last = self.timer.elapsed().as_secs_f64();
        self.update_data();

        let rotation_increment = 0.2;
        self.rotation += rotation_increment * since_last;


        // let mut mirror = self.data.clone();
        // mirror.reverse();
        // self.data.extend(mirror);

        
        // let mut graph = ayyeve_piston_ui::menu::menu_elements::Graph::new(
        //     Vector2::new(0.0, _args.window_size[1] - 500.0), 
        //     Vector2::new(500.0, 500.0),
        //     self.data.iter().map(|a|a.1).collect(),
        //     0.0, 20.0
        // );
        // list.extend(ayyeve_piston_ui::menu::menu_elements::ScrollableItem::draw(&mut graph, _args, Vector2::new(0.0, 0.0), depth));
        // list.push(Box::new(Rectangle::new(
        //     Color::WHITE,
        //     depth + 10.0,
        //     Vector2::new(0.0, _args.window_size[1] - 500.0), 
        //     Vector2::new(500.0, 500.0),
        //     None
        // )));


        let a = (2.0 * std::f64::consts::PI) / self.data.len() as f64;
        let n = (2.0 * std::f64::consts::PI * INNER_RADIUS) / self.data.len() as f64 / 2.0;

        for i in 0..self.data.len() {
            if self.data[i].1 <= CUTOFF {continue}

            let factor = (i as f64 + 2.0).log10();
            let l = INNER_RADIUS + self.data[i].1 as f64 * factor * self.bar_height;

            let theta = self.rotation + a * i as f64;
            let cos = theta.cos();
            let sin = theta.sin();
            let p1 = pos + Vector2::new(
                cos * INNER_RADIUS,
                sin * INNER_RADIUS
            );

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
