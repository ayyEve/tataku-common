
#[derive(Clone, Copy)]
pub struct Color {
    pub r:f32,
    pub g:f32,
    pub b:f32,
    pub a:f32,
}
// constant colors
#[allow(dead_code)]
impl Color {
    #[inline]
    pub const fn new(r:f32,g:f32,b:f32,a:f32) -> Self {Self{r,g,b,a}}

    pub const BLACK:Color = Color {r:0.0,g:0.0,b:0.0,a:1.0};
    pub const WHITE:Color = Color {r:1.0,g:1.0,b:1.0,a:1.0};

    // probably dont need black but w/e
    pub const TRANSPARENT_WHITE:Color = Color {r:0.0,g:0.0,b:0.0,a:0.0};
    pub const TRANSPARENT_BLACK:Color = Color {r:1.0,g:1.0,b:1.0,a:0.0};

    pub const RED:Color = Color {r:1.0,g:0.0,b:0.0,a:1.0};
    pub const BLUE:Color = Color {r:0.0,g:0.0,b:1.0,a:1.0};
    pub const GREEN:Color = Color {r:0.0,g:1.0,b:0.0,a:1.0};
    
    pub const YELLOW:Color = Color {r:1.0,g:1.0,b:0.0,a:1.0};
}


impl From<graphics::types::Color> for Color {
    fn from(c: graphics::types::Color) -> Self {
        Color {
            r:c[0],
            g:c[1],
            b:c[2],
            a:c[3]
        }
    }
}

impl Into<graphics::types::Color> for Color {
    fn into(self) -> graphics::types::Color {
        [self.r, self.g, self.b, self.a]
    }
}
