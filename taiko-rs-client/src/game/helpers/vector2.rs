use std::ops::{Add, Div, Mul, Neg, Sub};

pub struct Vector2 {
    x: f64,
    y: f64,
}
impl Vector2 {
    pub const fn new(x:f64, y: f64) -> Self {Self {x, y}}
    pub const fn zero() -> Self {Self {x:0.0, y:0.0}}
    pub const fn one() -> Self {Self {x:1.0, y:1.0}}
}

// negative nancy
impl Neg for Vector2 {
    type Output = Vector2;
    fn neg(self) -> Self::Output {
        Vector2::new(-self.x, -self.y)
    }
}


// come from
impl From<(f64,f64)> for Vector2 {
    fn from((x,y): (f64,f64)) -> Self {
        Vector2::new(x,y)
    }
}
impl From<[f64;2]> for Vector2 {
    fn from(a: [f64;2]) -> Self {
        Vector2::new(a[0],a[1])
    }
}

// goto
impl Into<(f64,f64)> for Vector2 {
    fn into(self) -> (f64,f64) {
        (self.x,self.y)
    }
}
impl Into<[f64;2]> for Vector2 {
    fn into(self) -> [f64;2] {
        [self.x, self.y]
    }
}


// add
// fuck you neb, i dont care if this isnt how math works
impl Add<f64> for Vector2 {
    type Output = Vector2;
    fn add(self, rhs: f64) -> Self::Output {
        Vector2::new(self.x + rhs, self.y + rhs)
    }
}
impl Add<Vector2> for Vector2 {
    type Output = Vector2;
    fn add(self, rhs: Vector2) -> Self::Output {
        Vector2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

// sub
impl Sub<f64> for Vector2 {
    type Output = Vector2;
    fn sub(self, rhs: f64) -> Self::Output {
        self + -rhs
    }
}
impl Sub<Vector2> for Vector2 {
    type Output = Vector2;
    fn sub(self, rhs: Vector2) -> Self::Output {
        self + -rhs
    }
}

// mul
impl Mul<f64> for Vector2 {
    type Output = Vector2;
    fn mul(self, rhs: f64) -> Self::Output {
        Vector2::new(self.x * rhs, self.y * rhs)
    }
}
impl Mul<Vector2> for Vector2 {
    type Output = Vector2;
    fn mul(self, rhs: Vector2) -> Self::Output {
        Vector2::new(self.x * rhs.x, self.y * rhs.y)
    }
}

// div
impl Div<f64> for Vector2 {
    type Output = Vector2;
    fn div(self, rhs: f64) -> Self::Output {
        Vector2::new(self.x / rhs, self.y / rhs)
    }
}
impl Div<Vector2> for Vector2 {
    type Output = Vector2;
    fn div(self, rhs: Vector2) -> Self::Output {
        Vector2::new(self.x / rhs.x, self.y / rhs.y)
    }
}

