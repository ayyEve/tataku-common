use opengl_graphics::{Texture, TextureSettings};

use crate::prelude::*;
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

    cookie: Image,

    ripples: Vec<TransformGroup>,
    last_ripple_at: f32,
    current_timing_point: TimingPoint
}
impl MenuVisualization {
    pub fn new() -> Self {
        Self {
            rotation: 0.0,
            data: Vec::new(),
            timer: Instant::now(),
            cookie: Image::new(Vector2::zero(), 0.0, Texture::empty(&TextureSettings::new()).unwrap(), Vector2::new(INNER_RADIUS, INNER_RADIUS)),

            bar_height: 1.0, //(Settings::get_mut().window_size[1] - INNER_RADIUS) / 128.0,

            // ripple things
            ripples: Vec::new(),
            last_ripple_at: 0.0,
            current_timing_point: TimingPoint::default()
        }
    }

    pub fn song_changed(&mut self) {
        self.last_ripple_at = 0.0;
        self.ripples.clear();
    }

    pub fn update(&mut self, manager: &mut Option<IngameManager>) {
        // update ripples
        if let Some(manager) = manager {
            let time = manager.time();

            let current_timing_point = manager.timing_point_at(time, false);
            let next_ripple = self.last_ripple_at + current_timing_point.beat_length;

            // timing point changed
            if current_timing_point.time != self.current_timing_point.time {
                self.last_ripple_at = 0.0;
                self.current_timing_point = current_timing_point.clone();
            }

            if self.last_ripple_at == 0.0 || time >= next_ripple {
                self.last_ripple_at = time;
                let mut group = TransformGroup::new();
                let duration = 1000.0;

                let mut circle = Circle::new(
                    Color::TRANSPARENT_BLACK,
                    -100000000000.0,
                    Settings::window_size() / 2.0,
                    INNER_RADIUS
                );
                circle.border = Some(Border::new(Color::WHITE, 2.0));
                group.items.push(DrawItem::Circle(circle));
                group.ripple(0.0, duration, time as f64, 5.0, true);

                // dm.transforms.push(Transformation::new(
                //     0.0,
                //     duration,
                //     TransformType::BorderTransparency {start: 1.0, end: 0.0},
                //     TransformEasing::EaseOutSine,
                //     time
                // ));
                // dm.transforms.push(Transformation::new(
                //     0.0,
                //     duration * 1.1,
                //     TransformType::Scale {start: 1.0, end: 5.0},
                //     TransformEasing::Linear,
                //     time
                // ));
                // dm.transforms.push(Transformation::new(
                //     0.0,
                //     duration * 1.1,
                //     TransformType::BorderSize {start: 2.0, end: 0.0},
                //     TransformEasing::EaseInSine,
                //     time
                // ));

                self.ripples.push(group);
            }
        
            let time = time as f64;
            self.ripples.retain_mut(|ripple| {
                ripple.update(time);
                ripple.items[0].visible()
            });
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

        for ripple in self.ripples.iter_mut() {
            ripple.draw(list)
        }


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


        let a = (2.0 * PI) / self.data.len() as f64;
        let n = (2.0 * PI * INNER_RADIUS) / self.data.len() as f64 / 2.0;

        for i in 0..self.data.len() {
            #[cfg(feature="bass_audio")]
            let val = self.data[i];
            #[cfg(feature="neb_audio")]
            let val = self.data[i].1;


            if val <= CUTOFF {continue}

            let factor = (i as f64 + 2.0).log10();
            let l = INNER_RADIUS + val as f64 * factor * self.bar_height;

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
