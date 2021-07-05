use crate::menu::ScrollableItem;



pub struct OnlineUser {
    
}
impl OnlineUser {
    
}

impl ScrollableItem for OnlineUser {
    fn size(&self) -> cgmath::Vector2<f64> {
        todo!()
    }

    fn get_tag(&self) -> String {
        todo!()
    }

    fn set_tag(&mut self, tag:&str) {
        todo!()
    }

    fn get_pos(&self) -> cgmath::Vector2<f64> {
        todo!()
    }

    fn set_pos(&mut self, pos:cgmath::Vector2<f64>) {
        todo!()
    }

    fn draw(&mut self, args:piston::RenderArgs, pos_offset:cgmath::Vector2<f64>) -> Vec<Box<dyn crate::render::Renderable>> {
        todo!()
    }

    fn on_click(&mut self, pos:cgmath::Vector2<f64>, button:piston::MouseButton) -> bool {
        todo!()
    }

    fn on_mouse_move(&mut self, pos:cgmath::Vector2<f64>) {
        todo!()
    }
}