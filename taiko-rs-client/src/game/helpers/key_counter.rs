use std::collections::HashMap;
use crate::game::get_font;
use crate::render::{Border, Color, Rectangle, Renderable, Vector2, Text};


const BOX_SIZE:Vector2 = Vector2::new(40.0, 40.0);

pub struct KeyCounter {
    pos: Vector2,
    keys_vec: Vec<piston::Key>,
    keys: HashMap<piston::Key, u16>
}
impl KeyCounter {
    pub fn new(keys_vec:Vec<piston::Key>, pos:Vector2) -> Self {
        let mut keys = HashMap::new();
        for key in keys_vec.iter() {keys.insert(*key, 0);}

        println!("{:?}", keys_vec);

        Self {
            keys,
            keys_vec,
            pos
        }
    }

    pub fn key_press(&mut self, key: piston::Key) {
        if self.keys.contains_key(&key) {
            *self.keys.get_mut(&key).unwrap() += 1;
        }
    }


    pub fn draw(&mut self, args: piston::RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        let font = get_font("main");
        let window_size:Vector2 = args.window_size.into();

        //TODO: center properly somehow
        for i in 0..self.keys_vec.len() {
            let pos = (Vector2::new(window_size.x - BOX_SIZE.x, window_size.y / 2.0 - BOX_SIZE.y) - self.pos) + Vector2::new(0.0, BOX_SIZE.y * i as f64);
            
            // draw bg box
            list.push(Box::new(Rectangle::new(
                Color::new(0.0, 0.0, 0.0, 0.8),
                -100.0,
                pos,
                BOX_SIZE,
                Some(Border::new(Color::BLACK, 2.0))
            )));

            let count = self.keys.get(&self.keys_vec[i]).unwrap();

            // draw key
            let mut text = Text::new(
                Color::WHITE,
                -100.1,
                pos,
                20,
                if *count == 0 {format!("{:?}", self.keys_vec[i])} else {format!("{}", count)},
                font.clone()
            );
            text.center_text(Rectangle::bounds_only(pos, BOX_SIZE));
            list.push(Box::new(text));
        }
    }
}
