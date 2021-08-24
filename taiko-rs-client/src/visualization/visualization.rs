use std::time::Instant;

use ayyeve_piston_ui::render::{Color, Line, Vector2};
use ayyeve_piston_ui::{menu::menu_elements::ScrollableItem, render::Renderable};



pub trait Visualization {
    fn should_lerp(&self) -> bool {true}
    fn draw(&mut self, args:piston::RenderArgs, list:&mut Vec<Box<dyn Renderable>>);
    fn update(&mut self) {}

    fn data(&mut self) -> &mut Vec<f32>;
    fn timer(&mut self) -> &mut Instant;
    fn update_data(&mut self) {
        // get the audio being fed to the sound card
        let audio_data = crate::game::audio::CURRENT_DATA.clone();
        let mut audio_data = audio_data.lock().clone();

        let n = 16384; // 8192
        let count = 720; // 80
        let audio_data = &mut audio_data[0..n];

        let plan = dft::Plan::new(dft::Operation::Forward, n);
        dft::transform(audio_data, &plan);

        let audio_data = audio_data[0..count].to_vec();
        let mut audio_data:Vec<f32> = audio_data
        .iter()
        .map(|i|i.abs())
        .collect();

        let time = self.timer();
        let elapsed = time.elapsed().as_secs_f32();
        *time = Instant::now();
        drop(time);


        let should_lerp = self.should_lerp();
        let data = self.data();
        if should_lerp && data.len() > 0 {
            for i in 0..audio_data.len() {
                audio_data[i] = lerp(data[i], audio_data[i], 20.0 * elapsed);
            }
        }

        *self.data() = audio_data;
    }
}

fn lerp(current:f32, target:f32, factor:f32) -> f32 {
    current + (target - current) * factor
}




use ayyeve_piston_ui::menu::menu_elements::Graph;

use crate::WINDOW_SIZE;

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

    fn draw(&mut self, args:piston::RenderArgs, list:&mut Vec<Box<dyn Renderable>>) {
        self.update_data();

        let depth = -100.0;

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

            let p1 = mid + Vector2::new(
                theta.cos() * inner_radius,
                theta.sin() * inner_radius
            );

            let l = inner_radius + self.data[i] as f64 * 10.0;
            let p2 = mid + Vector2::new(
                theta.cos() * l,
                theta.sin() * l
            );

            list.push(Box::new(Line::new(
                p1,
                p2,
                2.0,
                depth,
                Color::BLUE
            )));
        }

        // for i in 0..self.data.len() {
            
        // }
    }
}


