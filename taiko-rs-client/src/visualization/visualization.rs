use std::time::Instant;
use ayyeve_piston_ui::render::{Renderable, Vector2};

pub trait Visualization {
    fn should_lerp(&self) -> bool {true}
    fn draw(&mut self, args:piston::RenderArgs, pos_offset:Vector2, depth:f64, list:&mut Vec<Box<dyn Renderable>>);
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
