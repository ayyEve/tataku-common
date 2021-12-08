// opens on right hand side, should take up 1/3 of the screen, and is full height, or as tall as needed
use crate::prelude::*;


pub struct ChangelogDialog {
    items: Vec<String>,
    height: f64
}
impl ChangelogDialog {
    // pub fn new() -> Self {
    //     // assume the settings hasnt been updated
    //     let mut settings = Settings::get_mut("");
    //     // let last_update = settings.
    // }
}

impl Dialog<Game> for ChangelogDialog {
    fn get_bounds(&self) -> Rectangle {
        todo!()
    }

    fn draw(&mut self, args:&RenderArgs, depth: &f64, list: &mut Vec<Box<dyn Renderable>>) {
        todo!()
    }
}