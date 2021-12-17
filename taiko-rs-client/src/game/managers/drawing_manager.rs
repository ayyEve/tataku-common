use crate::prelude::*;

pub struct DrawingManager {
    pub items: Vec<DrawItem>,
    pub transforms: Vec<Transformation>
}
impl DrawingManager {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            transforms: Vec::new()
        }
    }

    pub fn update(&mut self, game_time: f64) {
        // going to need to figure out how to properly do this.
        // will need to store the original position somehow
        // alternatively, 
        // could apply these transforms to a clone of the arrays in the draw fn

        // though i think it might be best to redo these graphics structs
        // from use in taiko-rs

        // shouldnt be too terrible, just annoying tbh
        let mut transforms = std::mem::take(&mut self.transforms);
        transforms.retain(|transform| {
            let start_time = transform.start_time();
            // transform hasnt started, ignore
            if game_time >= start_time {
                let trans_val = transform.get_value(game_time);
                for i in self.items.iter_mut() {
                    i.apply_transform(transform, trans_val.clone())
                }
            }

            game_time < start_time + transform.duration
        });
        self.transforms = transforms;
    }

    //TODO: maybe this could be improved?
    pub fn draw(&mut self, list: &mut Vec<Box<dyn Renderable>>) {
        list.reserve(self.items.len());
        for i in self.items.iter() {
            if !i.visible() {continue}
            list.push(i.to_renderable());
        }
        // list.extend(&self.items);
    }
}

// fn to_boxed<R:'static+Transformable + Clone>(list:&Vec<R>) -> Vec<Box<dyn Renderable>> {
//     let mut output:Vec<Box<dyn Renderable>> = Vec::new();
//     for i in list {
//         output.push(Box::new(i.clone()))
//     }
//     output
// } 

#[derive(Clone)]
pub enum DrawItem {
    Line(Line),
    Text(Text),
    Image(Image),
    Circle(Circle),
    Rectangle(Rectangle),
    HalfCircle(HalfCircle),
}
impl DrawItem {
    pub fn apply_transform(&mut self, transform: &Transformation, trans_val: TransformValueResult) {
        match self {
            // DrawItem::Line(a) => a.apply_transform(transform, trans_val),
            // DrawItem::Text(a) => a.apply_transform(transform, trans_val),
            // DrawItem::Image(a) => a.apply_transform(transform, trans_val),
            DrawItem::Circle(a) => a.apply_transform(transform, trans_val),
            // DrawItem::Rectangle(a) => a.apply_transform(transform, trans_val),
            // DrawItem::HalfCircle(a) => a.apply_transform(transform, trans_val),
            _ => {}
        };
    }

    pub fn to_renderable(&self) -> Box<dyn Renderable> {
        let new_item:Box<dyn Renderable> = match self {
            DrawItem::Line(a) => Box::new(a.clone()),
            DrawItem::Text(a) => Box::new(a.clone()),
            DrawItem::Image(a) => Box::new(a.clone()),
            DrawItem::Circle(a) => Box::new(a.clone()),
            DrawItem::Rectangle(a) => Box::new(a.clone()),
            DrawItem::HalfCircle(a) => Box::new(a.clone()),
        };

        new_item
    }

    pub fn visible(&self) -> bool {
        match self {
            // DrawItem::Line(a) => a.apply_transform(transform, trans_val),
            // DrawItem::Text(a) => a.apply_transform(transform, trans_val),
            // DrawItem::Image(a) => a.apply_transform(transform, trans_val),
            DrawItem::Circle(a) => a.visible(),
            // DrawItem::Rectangle(a) => a.apply_transform(transform, trans_val),
            // DrawItem::HalfCircle(a) => a.apply_transform(transform, trans_val),
            _ => {true}
        }
    }
}