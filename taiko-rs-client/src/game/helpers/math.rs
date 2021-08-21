use std::collections::VecDeque;

use ayyeve_piston_ui::render::Vector2;

// this is essentially osu's math helper

pub const SLIDER_DETAIL_LEVEL:u32 = 50;


pub const E:f64 = 2.71828175;
pub const LOG2E:f64 = 1.442695;
pub const LOG10E:f64 = 0.4342945;
pub const PI:f64 = 3.14159274;
pub const TWO_PI:f64 = 6.28318548;
pub const PI_OVER_2:f64 = 1.57079637;
pub const PI_OVER_4:f64 = 0.7853982;

// const factorial_lookup:[f64;33] = [
//     1.0,
//     1.0,
//     2.0,
//     6.0,
//     24.0,
//     120.0,
//     720.0,
//     5040.0,
//     40320.0,
//     362880.0,
//     3628800.0,
//     39916800.0,
//     479001600.0,
//     6227020800.0,
//     87178291200.0,
//     1307674368000.0,
//     20922789888000.0,
//     355687428096000.0,
//     6402373705728000.0,
//     121645100408832000.0,
//     2432902008176640000.0,
//     51090942171709440000.0,
//     1124000727777607680000.0,
//     25852016738884976640000.0,
//     620448401733239439360000.0,
//     15511210043330985984000000.0,
//     403291461126605635584000000.0,
//     10888869450418352160768000000.0,
//     304888344611713860501504000000.0,
//     8841761993739701954543616000000.0,
//     265252859812191058636308480000000.0,
//     8222838654177922817725562880000000.0,
//     263130836933693530167218012160000000.0
// ];

// fn factorial(n:u32) -> f64 {
//     factorial_lookup[n as usize]
// }

// /// Evaluates the <a href="https://en.wikipedia.org/wiki/Binomial_coefficient">binomial coefficient</a> indexed by n and k.
// fn ni(n:u32, k:u32) -> f64 {
//     let a1 = factorial(n);
//     let a2 = factorial(k);
//     let a3 = factorial(n - k);
//     a1 / (a2 * a3)
// }

// /// Evaluates the i'th <a href="https://en.wikipedia.org/wiki/Bernstein_polynomial">bernstein polynomial</a> of degree
// /// n at position t.
// fn bernstein(n:u32, i:u32, t:f64) -> f64 {
//     let ti = if t == 0.0 && i == 0 {1.0} else {t.powi(i as i32)};
//     let tni = if n == i && t == 1.0 {1.0} else {(1.0 - t).powi(n as i32 - i as i32 )};
//     ni(n, i) * ti * tni
// }

// /// Creates a piecewise-linear approximation of a bezier curve, using bernstein polynomials to evaluate the curve at
// /// specific positions.
// fn create_bezier_bernstein(control_points:Vec<Vector2>) -> Vec<Vector2> {
//     let mut output = Vec::new();

//     let amount_output_points = SLIDER_DETAIL_LEVEL as usize * control_points.len();

//     let step = 1.0 / (amount_output_points - 1) as f64;
//     for i in 0..amount_output_points {
//         let t = step * i as f64;

//         let mut x = 0.0;
//         let mut y = 0.0;

//         for j in 0..control_points.len() {
//             let basis = bernstein(control_points.len() as u32 - 1, j as u32, t);
//             x += basis * control_points[j].x;
//             y += basis * control_points[j].y;
//         }

//         output.push(Vector2::new(x, y));
//     }

//     output
// }


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

        let mut subdivision_buffer1 = Vec::with_capacity(count);
        subdivision_buffer1.fill(Vector2::zero());

        let mut subdivision_buffer2 = Vec::with_capacity(count * 2 - 1);
        subdivision_buffer2.fill(Vector2::zero());

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
        for i in 1..control_points.len() {
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

        for i in 0..self.count {
            self.subdivision_buffer1[i] 
                = control_points[i].clone();
        }
        
        for i in 0..self.count {
            l[i] 
                = self.subdivision_buffer1[0];
            r[self.count - i - 1] 
                = self.subdivision_buffer1[self.count - i-1];
            
            for j in 0..self.count - i - 1 {
                self.subdivision_buffer1[j] = 
                    (self.subdivision_buffer1[j] 
                        + self.subdivision_buffer1[j + 1]) / 2.0
            }
        }
    }

    fn _subdivide_old(&mut self, control_points: &Vec<Vector2>, l: &mut Vec<Vector2>, r: &mut Vec<Vector2>) {
        let midpoints = &mut self.subdivision_buffer1;

        for i in 0..self.count {
            midpoints[i] = control_points[i].clone();
        }
        
        for i in 0..self.count {
            l[i] = midpoints[0];
            r[self.count - i - 1] = midpoints[self.count - i-1];
            
            for j in 0..self.count - i - 1 {
                midpoints[j] = (midpoints[j] + midpoints[j + 1]) / 2.0
            }
        }
    }

    
    /// This uses <a href="https://en.wikipedia.org/wiki/De_Casteljau%27s_algorithm">De Casteljau's algorithm</a> to obtain
    /// an optimal
    /// piecewise-linear approximation of the bezier curve with the same amount of points as there are control points.
    fn approximate(&mut self, control_points: &Vec<Vector2>, output: &mut Vec<Vector2>) {
        let mut l = self.subdivision_buffer1.clone();
        let mut r = self.subdivision_buffer2.clone();

        self.subdivide(&control_points, &mut l, &mut r);
        for i in 0..self.count {
            l[self.count + i] = r[i + 1];
        }

        output.push(control_points[0].clone());
        for i in 0..self.count - 1 {
            let index = i * 2;
            let p = (l[index - 1] + l[index] * 2.0 + l[index + 1]) * 0.25;
            output.push(p);
        }

        self.subdivision_buffer1 = l;
        self.subdivision_buffer2 = r;
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
                println!("approxmate: {:?}", &parent);
                self.approximate(&parent, &mut output);
                // freeBuffers.Push(parent);
                free_buffers.push_front(parent);
                continue;
            }

            // If we do not yet have a sufficiently "flat" (in other words, detailed) approximation we keep
            // subdividing the curve we are currently operating on.
            // Vector2[] rightChild = freeBuffers.Count > 0 ? freeBuffers.Pop() : new Vector2[this.count];
            let mut right_child = if free_buffers.len() > 0 {
                free_buffers.pop_front().unwrap()
            } else {
                vec![Vector2::zero(); self.count]
                // let mut v = Vec::with_capacity(self.count);
                // v.fill();
                // v
            };
            println!("not approxmate");
            // this.Subdivide(parent, leftChild, rightChild);
            self.subdivide(&parent, &mut left_child, &mut right_child);

            // We re-use the buffer of the parent for one of the children, so that we save one allocation per iteration.
            // for (int i = 0; i < this.count; ++i) parent[i] = leftChild[i];
            for i in 0..self.count {
                parent[i] = left_child[i]
            }

            // toFlatten.Push(rightChild);
            // toFlatten.Push(parent);
            to_flatten.push_front(right_child.clone());
            to_flatten.push_front(parent);
        }


        // output.Add(this.controlPoints[this.count - 1]);
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
    let mut working = Vec::with_capacity(count);
    let mut output = Vec::new();

    let points = SLIDER_DETAIL_LEVEL * count as u32;
    for iteration in 0..points+1 {
        for i in 0..count {working[i] = input[i]}
        for level in 0..count {
            for i in 0..count - level - 1 {
                // Vector2.Lerp(ref working[i], ref working[i + 1], (float)iteration / points, out working[i]);
                working[i] = lerp(working[i], working[i+1], (iteration / points) as f64);
            }
        }
        output.push(working[0]);
    }
    output
}

pub(crate) fn create_bezier_wrong(input: Vec<Vector2>) -> Vec<Vector2> {
    let count = input.len();
    let mut working = Vec::with_capacity(count);
    let mut output = Vec::new();

    let points = SLIDER_DETAIL_LEVEL * count as u32;
    for iteration in 0..points {
        for i in 0..count {working[i] = input[i]}
        for level in 0..count {
            for i in 0..count - level - 1 {
                // Vector2.Lerp(ref working[i], ref working[i + 1], (float)iteration / points, out working[i]);
                working[i] = lerp(working[i], working[i+1], (iteration / points) as f64);
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