use piston::RenderArgs;

use crate::{game::Vector2, menu::Menu, render::{Color, Line}};

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
    fn map_point(&self, point: f32) -> f64 {
        //
        let mapped = (point - self.min) / self.max; // TODO!

        self.pos.y + mapped as f64
    }
}

impl ScrollableItem for Graph {
    fn draw(&mut self, args:RenderArgs, pos_offset:Vector2, parent_depth:f64) -> Vec<Box<dyn crate::render::Renderable>> {
        let mut list: Vec<Box<dyn crate::render::Renderable>> = Vec::new();
        list.reserve(self.data_points.len());


        // TODO: cache this, probably better than recalcing it every time lmao
        let x_step = self.size.x / self.data_points.len() as f64;
        let mut prev_y = self.map_point(self.data_points[1]);

        list.push(Box::new(Line::new(
            Vector2::new(x_step * 0.0, self.map_point(self.data_points[0])),
            Vector2::new(x_step * 1.0, prev_y),
            1.5,
            parent_depth,
            Color::BLACK
        )));

        for i in 1..self.data_points.len() {
            let data = self.data_points[i];
            let new_y = self.map_point(data);

            list.push(Box::new(Line::new(
                Vector2::new(x_step * (i as f64 - 1.0), prev_y),
                Vector2::new(x_step * i as f64 , new_y),
                1.5,
                parent_depth,
                Color::BLACK
            )));
            prev_y = new_y;
        }

        list
    }

    fn size(&self) -> Vector2 {self.size}

    fn get_tag(&self) -> String {String::new()}
    fn set_tag(&mut self, tag:&str) {}

    fn get_pos(&self) -> Vector2 {self.pos}
    fn set_pos(&mut self, pos:Vector2) {self.pos = pos}
    fn get_selected(&self) -> bool {false}
    fn set_selected(&mut self, selected:bool) {}
    fn get_hover(&self) -> bool {false}
    fn set_hover(&mut self, hover:bool) {}
}
