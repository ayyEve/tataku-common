use dft::c32;
use std::time::Instant;
use ayyeve_piston_ui::render::{Renderable, Vector2};


pub trait Visualization {
    fn should_lerp(&self) -> bool {true}
    fn lerp_factor(&self) -> f32 {20.0}
    fn draw(&mut self, args:piston::RenderArgs, pos_offset:Vector2, depth:f64, list:&mut Vec<Box<dyn Renderable>>);
    fn update(&mut self) {}
    fn reset(&mut self) {}

    fn data(&mut self) -> &mut Vec<f32>;
    fn timer(&mut self) -> &mut Instant;
    fn update_data(&mut self) {
        // get the audio being fed to the sound card
        let audio_data = crate::game::audio::CURRENT_DATA.clone();
        let audio_data = audio_data.lock().clone();

        let audio_data = crate::game::audio::utils::deinterleave(&audio_data, 2)[0].clone();


        let n = audio_data.len();
        let count = n / 4; // was 960


        let mut audio_data = audio_data
            .iter()
            .map(|n| c32::new(*n, 1.0))
            .collect::<Vec<c32>>();

        // if n != audio_data.len() {
        //     audio_data = audio_data[0..n].to_vec();
        // }

        
        let plan = dft::Plan::new(dft::Operation::Forward, n);
        dft::transform(&mut audio_data, &plan);

        let audio_data = audio_data
            .iter()
            .map(|n| n.re)
            .collect::<Vec<f32>>();

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
        let factor = self.lerp_factor() * elapsed;
        let data = self.data();
        if should_lerp && data.len() > 0 {
            data.resize(audio_data.len(), 0.0);
            for i in 0..audio_data.len() {
                audio_data[i] = lerp(data[i], audio_data[i], factor);
            }
        }

        *self.data() = audio_data;
    }
}

fn lerp(current:f32, target:f32, factor:f32) -> f32 {
    current + (target - current) * factor
}
