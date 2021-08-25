use ayyeve_piston_ui::render::Vector2;

#[derive(Clone, Debug)]
pub struct NoteDef {
    pub pos: Vector2,
    pub time: f32,
    pub hitsound: u32,
    pub hitsamples: Vec<u8>
}


#[derive(Clone, Debug)]
pub struct SliderDef {
    // x,y,time,type,hitSound,curveType|curvePoints,slides,length,edgeSounds,edgeSets,hitSample
    pub pos: Vector2,
    pub time: f32,
    pub hitsound: u32,
    pub curve_type: CurveType,
    pub curve_points: Vec<Vector2>,
    pub slides: u64,
    pub length: f32,
    pub edge_sounds: Vec<u8>,
    pub edge_sets: Vec<u8>,
    
    pub hitsamples: Vec<u8>,

    pub raw_str: String
}


#[derive(Clone, Debug)]
pub struct SpinnerDef {
    pub pos: Vector2,
    pub time: f32,
    pub hitsound: u32,
    pub end_time: f32,
    
    pub hitsamples: Vec<u8>
}


#[derive(Clone, Debug)]
pub struct HoldDef {
    pub pos: Vector2,
    pub time: f32,
    pub hitsound: u32,
    pub end_time: f32,
    
    pub hitsamples: Vec<u8>
}



#[derive(Clone, Copy, Debug)]
pub enum CurveType {
    Bézier,
    Catmull,
    Linear,
    Perfect
}


/// only used for diff calc
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NoteType {
    Note,
    Slider,
    Spinner,
    /// mania only
    Hold
}