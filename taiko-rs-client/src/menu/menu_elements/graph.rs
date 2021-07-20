use piston::RenderArgs;

use crate::{game::Vector2, render::{Border, Circle, Color, Line, Rectangle}};
use super::ScrollableItem;

const LINE_WIDTH:f64 = 1.0;

pub struct Graph {
    pos: Vector2,
    size: Vector2,
    hover: bool,

    pub data_points: Vec<f32>,
    mapped_points: Vec<f64>,
    mouse_pos: Vector2,

    min: f32,
    max: f32,
}
impl Graph {
    pub fn new(pos: Vector2, size: Vector2, data_points: Vec<f32>, min: f32, max: f32) -> Self {
        let mapped_points = data_points.iter().map(|x| (max - x.clamp(min, max)) as f64 * size.y / (max - min).abs() as f64).collect();
        Self {
            pos,
            size,
            hover: false,

            data_points,
            mapped_points,
            mouse_pos: Vector2::one() * -10.0,
            min,
            max
        }
    }
}
impl ScrollableItem for Graph {
    fn draw(&mut self, _args:RenderArgs, pos_offset:Vector2, parent_depth:f64) -> Vec<Box<dyn crate::render::Renderable>> {
        let mut list: Vec<Box<dyn crate::render::Renderable>> = Vec::new();
        list.reserve(self.data_points.len() + 2);
        
        // background
        list.push(Box::new(Rectangle::new(
            Color::new(0.2, 0.2, 0.2, 0.7),
            parent_depth,
            self.pos + pos_offset,
            self.size,
            Some(Border::new(Color::RED, 1.5))
        )));
        // mid
        list.push(Box::new(Line::new(
            self.pos + pos_offset + Vector2::new(0.0, self.size.y / 2.0),
            self.pos + pos_offset + Vector2::new(self.size.x, self.size.y / 2.0),
            LINE_WIDTH,
            parent_depth,
            Color::WHITE
        )));

        // if theres no data points to draw, return
        if self.data_points.len() == 0 {return list}

        let mut prev_y = self.mapped_points[0];
        let x_step = self.size.x / self.data_points.len() as f64;

        for i in 1..self.mapped_points.len() {
            let new_y = self.mapped_points[i];
            list.push(Box::new(Line::new(
                self.pos + pos_offset + Vector2::new(x_step * (i-1) as f64, prev_y),
                self.pos + pos_offset + Vector2::new(x_step * i as f64, new_y),
                LINE_WIDTH,
                parent_depth + 1.0,
                Color::BLACK
            )));

            if self.get_hover() {
                // draw circles on the points
                list.push(Box::new(Circle::new(
                    Color::BLUE,
                    parent_depth + 0.5,
                    self.pos + pos_offset + Vector2::new(x_step * i as f64, new_y),
                    LINE_WIDTH * 2.0,
                )));
            }

            prev_y = new_y;
        }

        if self.get_hover() {
            // draw vertical line at mouse pos
            list.push(Box::new(Line::new(
                Vector2::new(self.mouse_pos.x, self.pos.y +pos_offset.y),
                Vector2::new(self.mouse_pos.x, self.pos.y + pos_offset.y + self.size.y),
                LINE_WIDTH,
                parent_depth - 1.0,
                Color::RED
            )));
        }

        list
    }
    fn on_mouse_move(&mut self, p:Vector2) {
        self.mouse_pos = p;
        self.check_hover(p);
    }

    fn size(&self) -> Vector2 {self.size}
    fn get_pos(&self) -> Vector2 {self.pos}
    fn set_pos(&mut self, pos:Vector2) {self.pos = pos}
    fn get_hover(&self) -> bool {self.hover}
    fn set_hover(&mut self, hover:bool) {self.hover = hover}

    fn get_tag(&self) -> String {String::new()}
    fn set_tag(&mut self, _tag:&str) {}
    fn get_selected(&self) -> bool {false}
    fn set_selected(&mut self, _selected:bool) {}
}
