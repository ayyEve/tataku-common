use ayyeve_piston_ui::render::{Color, Vector2};


#[derive(Copy, Clone, Default)]
pub struct Tranformation {
    /// how long to wait before this transform is started
    pub offset: f32,
    /// how long the tranform lasts
    pub duration: f32,
    pub trans_type: TransformType,
    pub easing_type: TransformEasing,
}

#[derive(Copy, Clone)]
pub enum TransformType {
    None, // default
    Position {start:Vector2, end: Vector2},
    Color {start: Color, end: Color},
}
impl Default for TransformType {
    fn default() -> Self {
        TransformType::None
    }
}


#[derive(Copy, Clone)]
pub enum TransformEasing {
    Linear,
}
impl Default for TransformEasing {
    fn default() -> Self {
        TransformEasing::Linear
    }
}