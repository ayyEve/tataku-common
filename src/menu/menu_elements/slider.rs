use std::ops::{Range, RangeBounds};

use cgmath::Vector2;

use crate::{game::get_font, menu::ScrollableItem, render::{Color, Rectangle, Renderable, Text, Border}};

// i think this is going to require me to implement mouse up events. rip
const SNAP_LENIENCY:f64 = 0.5;

#[derive(Clone)]
pub struct Slider {
    pos: Vector2<f64>,
    size: Vector2<f64>,
    hover:bool,
    tag: String,

    text: String,

    value: f64,
    range: Option<Range<f64>>,
    snapping:Option<f64>, // snap every multiple of this

    mouse_down:bool, // store the mouse state, because we dont have a hold event
}
impl Slider {
    pub fn new(pos: Vector2<f64>, size: Vector2<f64>, text:&str, value:f64, range:Option<Range<f64>>, snapping:Option<f64>) -> Slider {
        Slider {
            pos, 
            size, 
            hover: false,
            tag: String::new(),

            text: text.to_owned(),
            value,
            range,
            snapping,

            mouse_down:false,
        }
    }

    pub fn get_slider_bounds(&self) -> Rectangle {
        let font_size:u32 = 12;
        // draw text
        let txt = Text::new(
            Color::WHITE,
            0.0,
            Vector2::new(0.0,0.0),
            font_size,
            self.text.clone(),
            get_font("main")
        );
        let text_size = txt.measure_text();

        Rectangle::new(
            Color::BLACK,
            1.0,
            Vector2::new(text_size.x, 0.0),
            Vector2::new(self.size.x - text_size.x, self.size.y),
            None
        )
    }
}

impl ScrollableItem for Slider {
    fn size(&self) -> Vector2<f64> {self.size}
    fn get_pos(&self) -> Vector2<f64> {self.pos}
    fn set_pos(&mut self, pos:Vector2<f64>) {self.pos = pos}
    fn get_tag(&self) -> String {self.tag.clone()}
    fn set_tag(&mut self, tag:String) {self.tag = tag}
    fn get_value(&self) -> Box<dyn std::any::Any> {Box::new(self.value)}

    fn draw(&mut self, _args:piston::RenderArgs, pos_offset:Vector2<f64>) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        let font_size:u32 = 12;
        
        // draw bounding box
        list.push(Box::new(Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            0.0,
            self.pos+pos_offset,
            self.size,
            if self.hover {Some(Border::new(Color::RED, 1.0))} else {None}
        )));

        // draw text
        let txt = Text::new(
            Color::WHITE,
            -1.0,
            self.pos+pos_offset,
            font_size,
            self.text.clone(),
            get_font("main")
        );
        let text_size = txt.measure_text();
        list.push(Box::new(txt));

        // draw checkbox bounding box
        // list.push(Box::new(Rectangle::new(
        //     Color::TRANSPARENT_BLACK,
        //     1.0,
        //     self.pos+pos_offset,
        //     Vector2::new(self.size.y, self.size.y),
        //     if self.hover {Some(Border::new(Color::BLACK, 1.0))} else {None}
        // )));
        // if self.checked {
        //     list.push(Box::new(Rectangle::new(
        //         Color::YELLOW,
        //         0.0,
        //         self.pos+pos_offset + Vector2::new(INNER_BOX_PADDING, INNER_BOX_PADDING),
        //         Vector2::new(self.size.y-INNER_BOX_PADDING*2.0, self.size.y-INNER_BOX_PADDING * 2.0),
        //         None
        //     )));
        // }
        
        // draw track

        // draw snap lines

        // draw value bar



        list
    }

    fn on_click(&mut self, pos:Vector2<f64>, _button:piston::MouseButton) -> bool {
        if self.hover {
            // extrapolate value
            let mut val:f64;

            let bounds = self.get_slider_bounds();
            let rel_x = pos.x - (self.pos.x + bounds.pos.x + bounds.size.x);
            val = rel_x / bounds.size.x;

            // make sure its within bounds
            if let Some(range) = &self.range {
                val += range.start;
                val = val.clamp(range.start, range.end);
                println!("val:{}, min:{}, max:{}", val, range.start, range.end);
            }

            // check snapping?

            self.value = val;
        }
        self.hover
    }
    fn on_mouse_move(&mut self, pos:Vector2<f64>) {self.hover = self.hover(pos)}
}