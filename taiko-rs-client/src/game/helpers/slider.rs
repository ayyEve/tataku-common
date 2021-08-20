use ayyeve_piston_ui::render::Vector2;
use crate::gameplay::{Beatmap, CurveType};
use super::math::*;


pub struct Line {
    pub p1: Vector2,
    pub p2: Vector2,

    pub straight: bool,
    pub force_end: bool
}
impl Line {
    pub fn new(p1: Vector2, p2: Vector2) -> Self {
        Self {
            p1,
            p2,
            straight: false,
            force_end: false
        }
    }
}


// struct Curve {
//     curve_type: CurveType,
//     curve_length: f64,
//     points: Vec<Line>,
// }

pub fn get_curve_points(curve_type: CurveType, points:Vec<Vector2>, beatmap: Beatmap) -> Vec<Line> {
    let mut path = Vec::new();
    // let mut linear_spacing = 8;

    match curve_type {
        CurveType::Catmull => {
            for j in 0..points.len() {
                let v1 = if j >= 1 {points[j-1]} else {points[j]};
                let v2 = points[j];
                let v3 = if j + 1 < points.len() {points[j + 1]} else {v2 + (v2 - v1)};
                let v4 = if j + 1 < points.len() {points[j + 2]} else {v3 + (v3 - v2)};

                for k in 0..SLIDER_DETAIL_LEVEL {
                    path.push(Line::new(

                        catmull_rom(v1,v2,v3,v4, k as f64 / SLIDER_DETAIL_LEVEL as f64),
                        catmull_rom(v1,v2,v3,v4, (k + 1) as f64 / SLIDER_DETAIL_LEVEL as f64)
                    ));
                }
                // path[path.Count - 1].forceEnd = true;
            }
        }
        CurveType::Bézier => {
            let mut last_index = 0;

            let mut i = 0;
            while i < points.len() {
                if beatmap.metadata.beatmap_version > 8.0 {
                    let multipart_segment = (i as i32) < points.len() as i32 - 2 && points[i] == points[i + 1];

                    if multipart_segment || i == points.len() - 1 {
                        let this_length = points[last_index..i-last_index + 1].to_vec();
                        if this_length.len() == 2 {
                            //we can use linear algorithm for this segment
                            let l = Line::new(this_length[0], this_length[1]);
                            let segments = 1;
                            for j in 0..segments {
                                let mut line = Line::new(
                                    l.p1 + (l.p2 - l.p1) * (j as f64 / segments as f64),
                                    l.p1 + (l.p2 - l.p1) * ((j + 1) as f64 / segments as f64)
                                );
                                line.straight = true;
                                path.push(line);
                            }
                        } else {
                            if beatmap.metadata.beatmap_version < 10.0 {
                                //use the WRONG bezier algorithm. sliders will be 1/50 too short!
                                let points = create_bezier_wrong(this_length);
                                for j in 1..points.len() {
                                    path.push(Line::new(points[j - 1], points[j]));
                                }
                            } else {
                                //use the bezier algorithm
                                let points = create_bezier(this_length);
                                for j in 1..points.len() {
                                    path.push(Line::new(points[j - 1], points[j]));
                                }
                            }
                        }

                    }
                    let len = path.len();
                    path[len - 1].force_end = true;

                    //Need to skip one point since we consuned an extra.
                    if multipart_segment {i += 1}
                    last_index = i;

                } else if beatmap.metadata.beatmap_version > 6.0 {
                    let multipart_segment = (i as i32) < points.len() as i32 - 2 && points[i] == points[i + 1];

                    if multipart_segment || i == points.len() - 1 {
                        let this_length = points[last_index..i-last_index + 1].to_vec();
                        let points = create_bezier(this_length);
                        
                        for j in 1..points.len() {
                            path.push(Line::new(points[j-1], points[j]));
                        }
                        let len = path.len();
                        path[len - 1].force_end = true;
                        //Need to skip one point since we consuned an extra.
                        if multipart_segment {i += 1}
                        last_index = i;
                    }
                } else {
                    //This algorithm is broken for multipart sliders (http://osu.sifterapp.com/projects/4151/issues/145).
                    //Newer maps always use the one in the else clause.

                    if (i > 0 && points[i] == points[i - 1]) || i == points.len() - 1 {
                        let this_length = points[last_index..i-last_index + 1].to_vec();
                        let points = create_bezier(this_length);

                        for j in 1..points.len() {
                            path.push(Line::new(points[j-1], points[j]));
                        }
                        
                        path[path.len() - 1].force_end == true;
                        last_index = i;
                    }
                }
            
                i += 1;
            }
            
        }
        CurveType::Perfect => {
            // we may have 2 points when building the circle.
            if points.len() < 3 {
                return get_curve_points(CurveType::Linear, points, beatmap);
            }
            // more than 3 -> ignore them.
            if points.len() > 3 {
                return get_curve_points(CurveType::Bézier, points, beatmap);
            }
            let a = points[0];
            let b = points[1];
            let c = points[3];
            
            // all 3 points are on a straight line, avoid undefined behaviour:
            if is_straight_line(a,b,c) {
                return get_curve_points(CurveType::Linear, points, beatmap);
            }

            let (center, radius, t_initial, t_final) = circle_through_points(a,b,c);

            // this.curveLength = Math.Abs((t_final - t_initial) * radius);
            let curve_length = ((t_final - t_initial) * radius).abs();
            let segments = (curve_length * 0.125) as u32;
            let mut last_point = a;

            for i in 0..segments {
                let progress = i as f64 / segments as f64;
                let t = t_final * progress + t_initial * (1.0 - progress);
                let new_point = circle_point(center, radius, t);
                path.push(Line::new(last_point, new_point));
                last_point = new_point;
            }

            path.push(Line::new(last_point, c));
        }
        CurveType::Linear => {
            for i in 1..points.len() {
                let l = Line::new(points[i - 1], points[i]);
                let segments = 1;

                for j in 0..segments {
                    let mut l2 = Line::new(
                    l.p1 + (l.p2 - l.p1) * (j as f64 / segments as f64),
                    l.p1 + (l.p2 - l.p1) * ((j + 1) as f64 / segments as f64)
                    );
                    l2.straight = true;
                    path.push(l2);
                }
                let len = path.len();
                path[len - 1].force_end = true;
            }
        }
    }
    path
}

fn catmull_rom(value1:Vector2, value2:Vector2, value3:Vector2, value4:Vector2, amount:f64) -> Vector2 {
    let num = amount * amount;
    let num2 = amount * num;
    let mut result = Vector2::zero();

    result.x = 0.5 * (2.0 * value2.x + (-value1.x + value3.x) * amount + (2.0 * value1.x - 5.0 * value2.x + 4.0 * value3.x - value4.x) * num +
        (-value1.x + 3.0 * value2.x - 3.0 * value3.x + value4.x) * num2);

    result.y = 0.5 * (2.0 * value2.y + (-value1.y + value3.y) * amount + (2.0 * value1.y - 5.0 * value2.y + 4.0 * value3.y - value4.y) * num +
        (-value1.y + 3.0 * value2.y - 3.0 * value3.y + value4.y) * num2);
    return result;
}
