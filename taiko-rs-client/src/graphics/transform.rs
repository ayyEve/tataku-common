use crate::prelude::*;


#[derive(Copy, Clone, Default)]
pub struct Transformation {
    /// how long to wait before this transform is started
    pub offset: f64,
    /// how long the tranform lasts
    pub duration: f64,
    pub trans_type: TransformType,
    pub easing_type: TransformEasing,
    
    /// when was this transform crated? (ms)
    pub create_time: f64,
}
impl Transformation {
    pub fn new(offset: f64, duration: f64, trans_type: TransformType, easing_type: TransformEasing, create_time: f64) -> Self {
        Self {
            offset,
            duration,
            trans_type,
            easing_type,
            create_time
        }
    }
    pub fn start_time(&self) -> f64 {
        self.create_time + self.offset
    }
    
    pub fn get_value(&self, current_game_time: f64) -> TransformValueResult {
        // when this transform should start
        let begin_time = self.start_time();
        // how long has elapsed? (minimum 0ms, max self.duration)
        let elapsed = (current_game_time - begin_time).clamp(0.0, self.duration);

        // % for interpolation
        let factor = elapsed / self.duration;

        match self.trans_type {
            TransformType::Position { start, end } => 
                TransformValueResult::Vector2(self.run_easing_fn(end, start, factor)),

            TransformType::Rotation { start, end }
            | TransformType::Scale { start, end }  => 
                TransformValueResult::F64(self.run_easing_fn(end, start, factor)),

            TransformType::None => TransformValueResult::None,
            TransformType::Color { start, end } => todo!(),
        }
    }


    // thank god i got this working lmao
    pub fn run_easing_fn<I:Interpolation>(&self, target:I, current:I, factor: f64) -> I {
        match self.easing_type {
            TransformEasing::Linear => Interpolation::lerp(target, current, factor),

            TransformEasing::EaseInSine => Interpolation::easein_sine(target, current, factor),
            TransformEasing::EaseOutSine => Interpolation::easeout_sine(target, current, factor),
            TransformEasing::EaseInOutSine => Interpolation::easeinout_sine(target, current, factor),

            TransformEasing::EaseInQuadratic => Interpolation::easein_quadratic(target, current, factor),
            TransformEasing::EaseOutQuadratic => Interpolation::easeout_quadratic(target, current, factor),
            TransformEasing::EaseInOutQuadratic => Interpolation::easeinout_quadratic(target, current, factor),

            TransformEasing::EaseInCubic => Interpolation::easein_cubic(target, current, factor),
            TransformEasing::EaseOutCubic => Interpolation::easeout_cubic(target, current, factor),
            TransformEasing::EaseInOutCubic => Interpolation::easeinout_cubic(target, current, factor),

            TransformEasing::EaseInQuartic => Interpolation::easein_quartic(target, current, factor),
            TransformEasing::EaseOutQuartic => Interpolation::easeout_quartic(target, current, factor),
            TransformEasing::EaseInOutQuartic => Interpolation::easeinout_quartic(target, current, factor),

            TransformEasing::EaseInQuintic => Interpolation::easein_quintic(target, current, factor),
            TransformEasing::EaseOutQuintic => Interpolation::easeout_quintic(target, current, factor),
            TransformEasing::EaseInOutQuintic => Interpolation::easeinout_quintic(target, current, factor),

            TransformEasing::EaseInExponential => Interpolation::easein_exponential(target, current, factor),
            TransformEasing::EaseOutExponential => Interpolation::easeout_exponential(target, current, factor),
            TransformEasing::EaseInOutExponential => Interpolation::easeinout_exponential(target, current, factor),

            TransformEasing::EaseInCircular => Interpolation::easein_circular(target, current, factor),
            TransformEasing::EaseOutCircular => Interpolation::easeout_circular(target, current, factor),
            TransformEasing::EaseInOutCircular => Interpolation::easeinout_circular(target, current, factor),

            TransformEasing::EaseInBack(c1, c2) => Interpolation::easein_back(target, current, factor, c1, c2),
            TransformEasing::EaseOutBack(c1, c2) => Interpolation::easeout_back(target, current, factor, c1, c2),
            TransformEasing::EaseInOutBack(c1, c2) => Interpolation::easeinout_back(target, current, factor, c1, c2),
        }
    }
}


pub enum TransformValueResult {
    None,
    Vector2(Vector2),
    F64(f64),
    Color(Color)
}
impl Into<Vector2> for TransformValueResult {
    fn into(self) -> Vector2 {
        if let Self::Vector2(v) = self {
            v
        } else {
            // we want to crash here
            // if we get here its an issue in my code, and must be fixed
            panic!("NOT A VECTOR2!!")
        }
    }
}
impl Into<f64> for TransformValueResult {
    fn into(self) -> f64 {
        if let Self::F64(v) = self {
            v
        } else {
            // we want to crash here
            // if we get here its an issue in my code, and must be fixed
            panic!("NOT AN f64!!")
        }
    }
}



#[derive(Copy, Clone)]
pub enum TransformType {
    None, // default
    Position {start: Vector2, end: Vector2},
    Scale {start: f64, end: f64},
    Rotation {start: f64, end: f64},
    Color {start: Color, end: Color},
}
impl Default for TransformType {
    fn default() -> Self {
        TransformType::None
    }
}


/// values and equations taken from https://easings.net/
#[derive(Copy, Clone)]

pub enum TransformEasing {
    Linear,
    // sine
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    // quadratic
    EaseInQuadratic,
    EaseOutQuadratic,
    EaseInOutQuadratic,
    // cubic
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    // quartic
    EaseInQuartic,
    EaseOutQuartic,
    EaseInOutQuartic,
    // quintic
    EaseInQuintic,
    EaseOutQuintic,
    EaseInOutQuintic,
    // exponential
    EaseInExponential,
    EaseOutExponential,
    EaseInOutExponential,
    // circular
    EaseInCircular,
    EaseOutCircular,
    EaseInOutCircular,
    // back
    EaseInBack(f64, f64),
    EaseOutBack(f64, f64),
    EaseInOutBack(f64, f64),
}
impl Default for TransformEasing {
    fn default() -> Self {
        TransformEasing::Linear
    }
}

pub trait Transformable: Renderable {
    fn apply_transform(&mut self, transform: &Transformation, game_time:f64);
}