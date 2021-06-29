use std::ops::Range;

use cgmath::Vector2;

use crate::{game::get_font, menu::ScrollableItem, render::{Color, Rectangle, Renderable, Text, Border}};

// i think this is going to require me to implement mouse up events. rip
const SNAP_LENIENCY:f64 = 0.5;
const TRACKBAR_WIDTH:f64 = 1.0;

#[derive(Clone)]
pub struct Slider {
    pos: Vector2<f64>,
    size: Vector2<f64>,
    hover:bool,
    tag: String,

    text: String,

    value: f64,
    range: Range<f64>,
    snapping:Option<f64>, // snap every multiple of this

    mouse_down:bool, // store the mouse state, because we dont have a hold event
}
impl Slider {
    pub fn new(pos: Vector2<f64>, size: Vector2<f64>, text:&str, value:f64, range:Option<Range<f64>>, snapping:Option<f64>) -> Slider {
        let range = if let Some(r) = range{r} else {0.0..100.0};

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

    fn text_val(&self) -> String {format!("{}: {:.2}", self.text, self.value)}
    pub fn get_slider_bounds(&self) -> Rectangle {
        let font_size:u32 = 12;
        // draw text
        let txt = Text::new(
            Color::WHITE,
            0.0,
            Vector2::new(0.0, 0.0),
            font_size,
            format!("{}: {:.2}", self.text, self.range.end),
            get_font("main")
        );
        let text_size = txt.measure_text() + Vector2::new(10.0, 0.0);

        Rectangle::bounds_only(
            Vector2::new(text_size.x, 0.0),
            Vector2::new(self.size.x - text_size.x, self.size.y)
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
        let slider_bounds = self.get_slider_bounds();

        let mut txt = Text::new(
            Color::WHITE,
            -1.0,
            Vector2::new(0.0, 20.0),
            font_size,
            self.text_val(),
            get_font("main")
        );
        txt.center_text(Rectangle::bounds_only(self.pos+pos_offset, Vector2::new(self.size.x - slider_bounds.size.x, self.size.y)));
        //TODO: center text to area that slider_bounds isnt ([text][slider_bounds])
        // let text_size = txt.measure_text();
        list.push(Box::new(txt));
        
        // draw track
        list.push(Box::new(Rectangle::new(
            Color::BLACK,
            0.0,
            self.pos + pos_offset + slider_bounds.pos,
            slider_bounds.size,
            None
        )));

        // draw snap lines (definitely doesnt work yet)
        if let Some(snap) = self.snapping {
            for i in 0..slider_bounds.size.x.floor() as i32 {

                list.push(Box::new(Rectangle::new(
                    Color::RED,
                    -0.5,
                        // bounds offset
                    (self.pos + pos_offset + Vector2::new(slider_bounds.pos.x - TRACKBAR_WIDTH/2.0, 0.0))
                        // actual value offset
                        + Vector2::new(i as f64 * snap, 0.0),
                    Vector2::new(TRACKBAR_WIDTH, self.size.y / 1.3),
                    None
                )));
            }
        }

        // draw value bar
        list.push(Box::new(Rectangle::new(
            Color::BLUE,
            -1.0,
                // bounds offset
            (self.pos + pos_offset + Vector2::new(slider_bounds.pos.x - TRACKBAR_WIDTH/2.0, 0.0))
                // actual value offset
                + Vector2::new((self.value + self.range.start) / (self.range.end - self.range.start) * slider_bounds.size.x, 0.0),
            Vector2::new(TRACKBAR_WIDTH, self.size.y),
            None
        )));

        list
    }

    fn on_click(&mut self, pos:Vector2<f64>, _button:piston::MouseButton) -> bool {
        if self.hover {
            // extrapolate value
            let bounds = self.get_slider_bounds();
            let rel_x = pos.x - (self.pos.x + bounds.pos.x); // mouse pos - (self offset + text pos offset) (ie, mouse- slider bar start)
            let mut val = self.range.start + (rel_x / bounds.size.x) * (self.range.end - self.range.start);

            // solve for rel_x
            // val = min + (rel_x / bounds.x) * (max - min)
            // (val - min) = (rel_x / bounds.x) * (max - min)
            // (val - min) / (max - min) = rel_x / bounds.x
            // (val - min) / (max - min) * bounds.x = rel_x


            // check snapping (probably needs work lol)
            if let Some(snap) = self.snapping {
                if (val % snap).abs() < SNAP_LENIENCY {
                    //TODO: find out if the snap is "up" or "down"
                    println!("snapping");
                    val -= val % snap;
                }
            }

            // make sure its within bounds
            val = val.clamp(self.range.start, self.range.end);
            // println!("val:{}, min:{}, max:{}", val, self.range.start, self.range.end);

            self.value = val;
        }
        self.hover
    }
    fn on_mouse_move(&mut self, pos:Vector2<f64>) {self.hover = self.hover(pos)}
}