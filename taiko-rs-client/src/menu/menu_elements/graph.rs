use piston::RenderArgs;

use crate::{game::Vector2, render::{Border, Color, Line, Rectangle}};

use super::ScrollableItem;



pub struct Graph {
    pos: Vector2,
    size: Vector2,

    data_points: Vec<f32>,
    min: f32,
    max: f32,
    mid: f32,
}
impl Graph {

    pub fn new(pos: Vector2, size: Vector2, data_points: Vec<f32>, min: f32, mid: f32, max: f32) -> Self {
        Self {
            pos,
            size,
            data_points,
            min,
            max,
            mid
        }
    }
}

impl ScrollableItem for Graph {
    fn draw(&mut self, _args:RenderArgs, pos_offset:Vector2, parent_depth:f64) -> Vec<Box<dyn crate::render::Renderable>> {
        let mut list: Vec<Box<dyn crate::render::Renderable>> = Vec::new();
        list.reserve(self.data_points.len());
        list.push(Box::new(Rectangle::new(
            Color::TRANSPARENT_WHITE,
            parent_depth,
            self.pos,
            self.size,
            Some(Border::new(Color::RED, 1.5))
        )));
        let colors = [
            Color::RED,
            Color::BLUE,
            Color::GREEN
        ];

        let data_points = self.data_points.iter().map(|x| (self.max - x.clone()) as f64 * self.size.y / (self.max - self.min).abs() as f64);
        let data_points:Vec<f64> = data_points.collect();
        let mut prev_y = data_points[0];
        let x_step = self.size.x / self.data_points.len() as f64;

        for i in 1..data_points.len() {
            // let data = self.data_points[i];
            let new_y = data_points[i];

            list.push(Box::new(Line::new(
                self.pos + pos_offset + Vector2::new(x_step * (i as f64 - 1.0), prev_y),
                self.pos + pos_offset + Vector2::new(x_step * i as f64, new_y),
                1.5,
                parent_depth,
                colors[i%colors.len()]
            )));
            prev_y = new_y;
        }

        list
    }

    fn size(&self) -> Vector2 {self.size}

    fn get_tag(&self) -> String {String::new()}
    fn set_tag(&mut self, _tag:&str) {}

    fn get_pos(&self) -> Vector2 {self.pos}
    fn set_pos(&mut self, pos:Vector2) {self.pos = pos}
    fn get_selected(&self) -> bool {false}
    fn set_selected(&mut self, _selected:bool) {}
    fn get_hover(&self) -> bool {false}
    fn set_hover(&mut self, _hover:bool) {}
}

pub fn map_range_f32(val:f32, val_min:f32, val_max:f32, min:f32, mid:f32, max:f32) -> f32 {
    let range_mid = val_min + (val_max - val_min) / 2.0;

    println!("val:{}, val_min:{}, val_max:{}, range_mid:{}, min:{}, mid:{}, max:{}", val, val_min,val_max,range_mid,  min, mid, max);

    let t = if range_mid == 0.0 {
        if val > 0.0 {
            mid + (max - mid) * val
        } else if val < 0.0 {
            mid - (mid - min) * -val
        } else {
            mid
        }
    } else {
        if val > range_mid {
            mid + (max - mid) * (val - range_mid) / range_mid
        } else if val < range_mid {
            mid - (mid - min) * (range_mid - val) / range_mid
        } else {
            mid
        }
    };
    
    println!("got {}", t);
    t
}