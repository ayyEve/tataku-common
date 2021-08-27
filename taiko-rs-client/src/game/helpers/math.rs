use std::collections::VecDeque;
use ayyeve_piston_ui::render::Vector2;

// this is essentially osu's math helper

pub const SLIDER_DETAIL_LEVEL:u32 = 50;

pub const PI:f64 = 3.14159274;
pub const TWO_PI:f64 = 6.28318548;

const BEZIER_TOLERANCE:f64 = 0.5;
const BEZIER_TOLERANCE_SQ:f64 = BEZIER_TOLERANCE * BEZIER_TOLERANCE;

struct BezierApproximator {
    count: usize,
    control_points: Vec<Vector2>,
    subdivision_buffer1: Vec<Vector2>,
    subdivision_buffer2: Vec<Vector2>
}
impl BezierApproximator {
    fn new(control_points: Vec<Vector2>) -> Self {
        let count = control_points.len();

        let mut subdivision_buffer1 = Vec::new();
        for _ in 0..count {
            subdivision_buffer1.push(Vector2::zero());
        }

        let mut subdivision_buffer2 = Vec::new();
        for _ in 0..count * 2 - 1 {
            subdivision_buffer2.push(Vector2::zero());
        }

        Self {
            control_points,
            count,

            subdivision_buffer1,
            subdivision_buffer2,
        }
    }

    
    /// Make sure the 2nd order derivative (approximated using finite elements) is within tolerable bounds.
    /// NOTE: The 2nd order derivative of a 2d curve represents its curvature, so intuitively this function
    /// checks (as the name suggests) whether our approximation is _locally_ "flat". More curvy parts
    /// need to have a denser approximation to be more "flat".
    fn is_flat_enough(control_points: &Vec<Vector2>) -> bool {
        for i in 1..control_points.len()-1 {
            if length_squared(control_points[i-1] - control_points[i] * 2.0 + control_points[i+1]) > BEZIER_TOLERANCE_SQ {
                return false;
            }
        }
        true
    }

    
    /// Subdivides n control points representing a bezier curve into 2 sets of n control points, each
    /// describing a bezier curve equivalent to a half of the original curve. Effectively this splits
    /// the original curve into 2 curves which result in the original curve when pieced back together.
    fn subdivide(&mut self, control_points: &Vec<Vector2>, l: &mut Vec<Vector2>, r: &mut Vec<Vector2>) {
        let midpoints = &mut self.subdivision_buffer1;

        for i in 0..self.count {
            midpoints[i] = control_points[i];
        }
        
        for i in 0..self.count {
            l[i] = midpoints[0];
            r[self.count - i - 1] = midpoints[self.count - i - 1];
            
            for j in 0..self.count - i - 1 {
                midpoints[j] = (midpoints[j] + midpoints[j + 1]) / 2.0
            }
        }
    }

    /// This uses <a href="https://en.wikipedia.org/wiki/De_Casteljau%27s_algorithm">De Casteljau's algorithm</a> to obtain
    /// an optimal piecewise-linear approximation of the bezier curve with the same amount of points as there are control points.
    fn approximate(&mut self, control_points: &Vec<Vector2>, output: &mut Vec<Vector2>) {
        let mut l = self.subdivision_buffer2.clone();
        let mut r = self.subdivision_buffer1.clone();

        self.subdivide(&control_points, &mut l, &mut r);

        for i in 0..self.count - 1 {
            l[self.count + i] = r[i + 1];
        }

        output.push(control_points[0]);
        for i in 1..self.count - 1 {
            let index = i * 2;
            let p = (l[index - 1] + l[index] * 2.0 + l[index + 1]) * 0.25;
            output.push(p);
        }

        self.subdivision_buffer2 = l;
        self.subdivision_buffer1 = r;
    }

    
    /// Creates a piecewise-linear approximation of a bezier curve, by adaptively repeatedly subdividing
    /// the control points until their approximation error vanishes below a given threshold.
    fn create_bezier(&mut self) -> Vec<Vector2> {
        let mut output = Vec::new();
        if self.count == 0 {return output}

        // Stack<Vector2[]> toFlatten = new Stack<Vector2[]>();
        // Stack<Vector2[]> freeBuffers = new Stack<Vector2[]>();
        let mut to_flatten = VecDeque::new();
        let mut free_buffers = VecDeque::new();


        // "toFlatten" contains all the curves which are not yet approximated well enough.
        // We use a stack to emulate recursion without the risk of running into a stack overflow.
        // (More specifically, we iteratively and adaptively refine our curve with a 
        // <a href="https://en.wikipedia.org/wiki/Depth-first_search">Depth-first search</a>
        // over the tree resulting from the subdivisions we make.)
        // toFlatten.Push(this.controlPoints.ToArray());
        to_flatten.push_front(self.control_points.clone());

        // Vector2[] leftChild = this.subdivisionBuffer2;
        let mut left_child = self.subdivision_buffer2.clone();
        
        while to_flatten.len() > 0 {
            let mut parent = to_flatten.pop_front().unwrap();

            if BezierApproximator::is_flat_enough(&parent) {
                // If the control points we currently operate on are sufficiently "flat", we use
                // an extension to De Casteljau's algorithm to obtain a piecewise-linear approximation
                // of the bezier curve represented by our control points, consisting of the same amount
                // of points as there are control points.

                // this.Approximate(parent, output);
                self.approximate(&parent, &mut output);
                free_buffers.push_front(parent);
                continue;
            }

            // If we do not yet have a sufficiently "flat" (in other words, detailed) approximation we keep
            // subdividing the curve we are currently operating on.
            let mut right_child = if free_buffers.len() > 0 {
                free_buffers.pop_front().unwrap()
            } else {
                vec![Vector2::zero(); self.count]
            };
            self.subdivide(&parent, &mut left_child, &mut right_child);

            // We re-use the buffer of the parent for one of the children, so that we save one allocation per iteration.
            for i in 0..self.count {
                parent[i] = left_child[i]
            }

            to_flatten.push_front(right_child.clone());
            to_flatten.push_front(parent);
        }

        output.push(self.control_points[self.count - 1]);
        output
    }
}

pub(crate) fn create_bezier(input: Vec<Vector2>) -> Vec<Vector2> {
    let mut b = BezierApproximator::new(input);
    b.create_bezier()
}

pub(crate) fn create_bezier_old(input: Vec<Vector2>) -> Vec<Vector2> {
    let count = input.len();
    let mut working = Vec::new();
    for _ in 0..count {working.push(Vector2::zero())}
    let mut output = Vec::new();

    let points = SLIDER_DETAIL_LEVEL * count as u32;
    for iteration in 0..points+1 {
        for i in 0..count {working[i] = input[i]}
        for level in 0..count {
            for i in 0..count - level - 1 {
                working[i] = lerp(working[i], working[i+1], iteration as f64 / points as f64);
            }
        }
        output.push(working[0]);
    }
    output
}

pub(crate) fn create_bezier_wrong(input: Vec<Vector2>) -> Vec<Vector2> {
    let count = input.len();

    let mut working = Vec::new();
    for _ in 0..count {working.push(Vector2::zero())}
    let mut output = Vec::new();

    let points = SLIDER_DETAIL_LEVEL * count as u32;

    for iteration in 0..points {
        for i in 0..count {working[i] = input[i]}

        for level in 0..count {
            for i in 0..count - level - 1 {
                working[i] = lerp(working[i], working[i+1], iteration as f64 / points as f64);
            }
        }
        output.push(working[0]);
    }

    output
}


fn length_squared(p:Vector2) -> f64 {
    p.x * p.x + p.y * p.y
}

fn lerp(value1: Vector2, value2: Vector2, amount:f64) -> Vector2 {
    Vector2::new(
        value1.x + (value2.x - value1.x) * amount,
        value1.y + (value2.y - value1.y) * amount
    )
}

fn distance(p1:Vector2, p2:Vector2) -> f64 {
    let num = p1.x - p2.x;
    let num2 = p1.y - p2.y;
    let num3 = num * num + num2 * num2;
    num3.sqrt()
}

pub fn is_straight_line(a:Vector2, b:Vector2, c:Vector2) -> bool {
    (b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y) == 0.0
}

pub fn circle_t_at(p:Vector2, c:Vector2) -> f64 {
    (p.y - c.y).atan2(p.x - c.x)
}

/// Circle through 3 points
/// http://en.wikipedia.org/wiki/Circumscribed_circle#Cartesian_coordinates
pub fn circle_through_points(a:Vector2, b:Vector2, c:Vector2) -> (Vector2, f64, f64, f64) {
    let d = (a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y)) * 2.0;
    let a_mag_sq = length_squared(a);
    let b_mag_sq = length_squared(b);
    let c_mag_sq = length_squared(c);

    let center = Vector2::new(
        (a_mag_sq * (b.y - c.y) + b_mag_sq * (c.y - a.y) + c_mag_sq * (a.y - b.y)) / d, 
        (a_mag_sq * (c.x - b.x) + b_mag_sq * (a.x - c.x) + c_mag_sq * (b.x - a.x)) / d
    );
    let radius = distance(center, a);

    let t_initial = circle_t_at(a, center);
    let mut t_mid = circle_t_at(b, center);
    let mut t_final = circle_t_at(c, center);

    while t_mid < t_initial {t_mid += TWO_PI}
    while t_final < t_initial {t_final += TWO_PI}
    if t_mid > t_final {t_final -= TWO_PI}

    (center, radius, t_initial, t_final)
}


pub(crate) fn circle_point(center:Vector2, radius:f64, a:f64) -> Vector2 {
    Vector2::new(
        a.cos() * radius,
        a.sin() * radius
    ) + center
}