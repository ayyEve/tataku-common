// use dft::c32;
use std::time::Instant;

use crate::game::Audio;
// use crate::game::fft::FFT;
use crate::render::{Renderable, Vector2};


pub use f32 as Amplitude;
pub use f32 as Frequency;

pub type FFTEntry = f32; //(Frequency, Amplitude);


pub trait Visualization {
    fn should_lerp(&self) -> bool {true}
    fn lerp_factor(&self) -> f32 {20.0}
    fn draw(&mut self, args:piston::RenderArgs, pos_offset:Vector2, depth:f64, list:&mut Vec<Box<dyn Renderable>>);
    fn update(&mut self) {}
    fn reset(&mut self) {}

    fn data(&mut self) -> &mut Vec<FFTEntry>;
    fn timer(&mut self) -> &mut Instant;
    fn update_data(&mut self) {
        // get the audio being fed to the sound card
        // let audio_data = crate::game::audio::CURRENT_DATA.clone();
        // let mut audio_data = audio_data.lock().clone();
        // // println!("{}", audio_data.len());

        // let len = audio_data.len();
        // let size;

        // if !cfg!(target_os = "linux") {
        //     let scale = (1024.0 / len as f32) * 8.0;
        //     for sample in audio_data.iter_mut() {
        //         *sample *= scale;
        //     }
        //     audio_data.resize(1024, 0.0);
        //     size = FFT::F1024;
        // } else {
        //     audio_data.resize(8192, 0.0);
        //     size = FFT::F8192;
        // }

        // let mut audio_data = crate::game::audio::fft::fft(
        //     &mut audio_data, 
        //     size
        // );

        // audio_data.retain(|(freq, _amp)| *freq < 7_000.0);

        // audio_data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // let mut audio_data = audio_data.iter().map(|(_freq, amp)| {
        //     *amp
        // }).collect::<Vec<f32>>();


        // let n = audio_data.len();
        // let count = n / 4; // was 960


        // let mut audio_data = audio_data
        //     .iter()
        //     .map(|n| c32::new(*n, 1.0))
        //     .collect::<Vec<c32>>();

        // // if n != audio_data.len() {
        // //     audio_data = audio_data[0..n].to_vec();
        // // }
        
        // let plan = dft::Plan::new(dft::Operation::Forward, n);
        // dft::transform(&mut audio_data, &plan);

        // let audio_data = audio_data
        //     .iter()
        //     .map(|n| n.re)
        //     .collect::<Vec<f32>>();

        // let audio_data = audio_data[0..count].to_vec();
        // let mut audio_data:Vec<f32> = audio_data
        //     .iter()
        //     .map(|i|i.abs())
        //     .collect();

        // println!("getting data");
        let mut audio_data = match Audio::get_song() {
            Some(stream) => stream.get_data(bass::prelude::DataType::FFT2048, 1024i32),
            None => return
        }.unwrap_or(vec![0.0]);


        let time = self.timer();
        let elapsed = time.elapsed().as_secs_f32();
        *time = Instant::now();
        drop(time);


        const MULT:f32 = 1_000.0;

        let should_lerp = self.should_lerp();
        let factor = self.lerp_factor() * elapsed;
        let data = self.data();
        if should_lerp && data.len() > 0 {
            data.resize(audio_data.len(), 0.0);
            for i in 0..audio_data.len() {
                audio_data[i] = lerp(data[i], audio_data[i] * MULT, factor);
            }
        }

        *self.data() = audio_data;
    }
}

fn lerp(current:f32, target:f32, factor:f32) -> f32 {
    current + (target - current) * factor
}
