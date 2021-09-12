use crate::Vector2;
use crate::helpers::math::*;
use crate::gameplay::{Beatmap, defs::{CurveType, SliderDef}};

#[derive(Copy, Clone, Debug)]
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

    pub fn rho(&self) -> f32 {
        length(self.p2 - self.p1)
    }
}


fn length(p:Vector2) -> f32 {
    let num = p.x * p.x + p.y*p.y;
    num.sqrt() as f32
}

#[derive(Clone)]
pub struct Curve {
    pub slider: SliderDef,
    pub path: Vec<Line>,
    pub end_time: f32,

    pub smooth_lines: Vec<Line>,
    pub cumulative_lengths: Vec<f32>,

    pub velocity: f32,
    pub score_times: Vec<f32>
}
impl Curve {
    fn new(slider: SliderDef, path: Vec<Line>, beatmap: &Beatmap) -> Self {
        // let l = slider.length * 1.4 * slider.slides as f32;
        // let v2 = 100.0 * beatmap.metadata.slider_multiplier * 1.4;
        let l = slider.length * slider.slides as f32;
        let v2 = 100.0 * beatmap.metadata.slider_multiplier;
        let bl = beatmap.beat_length_at(slider.time, true);
        let end_time = slider.time + (l / v2 * bl) - 1.0;

        let velocity = beatmap.slider_velocity_at(slider.time);
        Self {
            path,
            slider,
            velocity,
            end_time,
            smooth_lines: Vec::new(),
            cumulative_lengths: Vec::new(),
            score_times: Vec::new()
        }
    }

    pub fn time_at_length(&self, length:f32) -> f32 {
        self.slider.time + (length / self.velocity) * 1000.0
    }

    pub fn length(&self) -> f32 {
        self.end_time - self.slider.time
    }

    pub fn position_at_time(&self, time:f32) -> Vector2 {
        // if (this.sliderCurveSmoothLines == null) this.UpdateCalculations();
        if time < self.slider.time || time > self.end_time {return self.slider.pos}

        let mut pos = (time - self.slider.time) / (self.length() / self.slider.slides as f32);
        if pos % 2.0 > 1.0 {
            pos = 1.0 - (pos % 1.0);
        } else {
            pos = pos % 1.0;
        }

        let length_required = self.cumulative_lengths.last().unwrap() * pos;
        self.position_at_length(length_required)
    }

    pub fn position_at_length(&self, length:f32) -> Vector2 {
        // if (this.sliderCurveSmoothLines == null || this.cumulativeLengths == null) this.UpdateCalculations();
        if self.smooth_lines.len() == 0 || self.cumulative_lengths.len() == 0 {return self.slider.pos}
        
        if length == 0.0 {return self.smooth_lines[0].p1}
        
        let end = *self.cumulative_lengths.last().unwrap();

        if length > end {
            let end = self.smooth_lines.len();
            return self.smooth_lines[end - 1].p2;
        }
        let i = match self.cumulative_lengths.binary_search_by(|f| f.partial_cmp(&length).unwrap()) {
            Ok(n) => n,
            Err(n) => n.min(self.cumulative_lengths.len() - 1),
        };

        let length_next = self.cumulative_lengths[i];
        let length_previous = if i == 0 {0.0} else {self.cumulative_lengths[i - 1]};
        
        let mut res = self.smooth_lines[i].p1;
    
        if length_next != length_previous {
            let n = (self.smooth_lines[i].p2 - self.smooth_lines[i].p1) 
                * ((length - length_previous) / (length_next - length_previous)) as f64;
            res = res + n;
        }

        res
    }
}


pub fn get_curve(slider:&SliderDef, beatmap: &Beatmap) -> Curve {
    let mut points = slider.curve_points.clone();
    points.insert(0, slider.pos);
    let mut path = Vec::new();

    match slider.curve_type {
        CurveType::Catmull => {
            for j in 0..points.len() {
                let v1 = if j >= 1 {points[j-1]} else {points[j]};
                let v2 = points[j];
                let v3 = if j + 1 < points.len() {points[j + 1]} else {v2 + (v2 - v1)};
                let v4 = if j + 2 < points.len() {points[j + 2]} else {v3 + (v3 - v2)};

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
                if beatmap.metadata.beatmap_version > 8 {
                    let multipart_segment = (i as i32) < points.len() as i32 - 2 && points[i] == points[i + 1];
                    // println!("i: {}, p.len(): {}", i, points.len());

                    if multipart_segment || i == points.len() - 1 {
                        let this_length = points[last_index..i+1].to_vec();
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
                            if beatmap.metadata.beatmap_version < 10 {
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
                        // let len = path.len();
                        // if len > 1 {
                        //     path[len - 1].force_end = true;
                        // }

                        //Need to skip one point since we consuned an extra.
                        if multipart_segment {i += 1}
                        last_index = i;
                    }
                    

                } else if beatmap.metadata.beatmap_version > 6 {
                    let multipart_segment = (i as i32) < points.len() as i32 - 2 && points[i] == points[i + 1];

                    if multipart_segment || i == points.len() - 1 {
                        let this_length = points[last_index..i + 1].to_vec();
                        let points = create_bezier_old(this_length);
                        
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
                        let this_length = points[last_index..i + 1].to_vec();
                        let points = create_bezier_wrong(this_length);

                        for j in 1..points.len() {
                            path.push(Line::new(points[j-1], points[j]));
                        }
                        
                        let len = path.len();
                        path[len - 1].force_end = true;
                        last_index = i;
                    }
                }
            
                i += 1;
            }
        }
        CurveType::Perfect => {
            // we may have 2 points when building the circle.
            if points.len() < 3 {
                let mut slider = slider.clone();
                slider.curve_type = CurveType::Linear;
                return get_curve(&slider, beatmap);
            }
            // more than 3 -> ignore them.
            if points.len() > 3 {
                let mut slider = slider.clone();
                slider.curve_type = CurveType::Bézier;
                return get_curve(&slider, beatmap);
            }
            let a = points[0];
            let b = points[1];
            let c = points[2];
            
            // all 3 points are on a straight line, avoid undefined behaviour:
            if is_straight_line(a,b,c) {
                let mut slider = slider.clone();
                slider.curve_type = CurveType::Linear;
                return get_curve(&slider, beatmap);
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
    


    let mut curve = Curve::new(slider.clone(), path, beatmap);
    let slider_scoring_point_distance = 100.0 * (beatmap.metadata.slider_multiplier / beatmap.metadata.slider_tick_rate);
    let tick_distance;
    if beatmap.metadata.beatmap_version < 8 {
        tick_distance = slider_scoring_point_distance;
    } else {
        tick_distance = slider_scoring_point_distance / beatmap.bpm_multiplier_at(curve.slider.time);
    }

    let path_count = curve.path.len();
    let mut total = 0.0;
    if path_count > 0 {
        //fill the cache
        curve.smooth_lines = curve.path.clone();
        curve.cumulative_lengths.clear();

        for l in 0..curve.path.len() {
            total += curve.path[l].rho();
            curve.cumulative_lengths.push(total);
        }
    }

    if path_count < 1 {return curve}

    // let mut first_run = true;
    let mut scoring_length_total = 0.0;
    // let mut current_time = curve.slider_def.time; // self.start_time;
    // let mut p1 = Vector2::zero();
    // let mut p2 = Vector2::zero();
    let mut scoring_distance = 0.0;

    // self.
    // let position2 = curve.path[path_count - 1].p2;

    for i in 0..curve.slider.slides as usize {
        let mut distance_to_end = total;
        let mut skip_tick = false;
        // let reverse_start_time = current_time as i32;
        let min_tick_distance_from_end = 0.01 * curve.velocity;

        let reverse = (i % 2) == 1;
        let start = if reverse {path_count as i32 - 1} else {0};
        let end = if reverse {-1} else {path_count as i32};
        let direction:i32 = if reverse {-1} else {1};

        let mut j = start;
        while j < end {
            // let l = curve.path[j as usize];

            // float distance = (float)(this.cumulativeLengths[j] - (j == 0 ? 0 : this.cumulativeLengths[j - 1])); ;
            let distance = curve.cumulative_lengths[j as usize] - if j == 0 {0.0} else {curve.cumulative_lengths[j as usize - 1]};
            
            // if reverse {
            //     p1 = l.p2;
            //     p2 = l.p1;
            // } else {
            //     p1 = l.p1;
            //     p2 = l.p2;
            // }

            // let duration = 1000.0 * distance / curve.velocity;

            // current_time += duration;
            scoring_distance += distance;

            while scoring_distance >= tick_distance && !skip_tick {
                scoring_length_total += tick_distance;
                scoring_distance -= tick_distance;
                distance_to_end -= tick_distance;

                skip_tick = distance_to_end <= min_tick_distance_from_end;
                if skip_tick {break}

                let score_time = curve.time_at_length(scoring_length_total);
                curve.score_times.push(score_time);

            }

            j += direction;
        }
    
        scoring_length_total += scoring_distance;
        let t = curve.time_at_length(scoring_length_total);
        curve.score_times.push(t);

        if skip_tick {
            scoring_distance = 0.0;
        } else {
            scoring_length_total -= tick_distance - scoring_distance;
            scoring_distance = tick_distance - scoring_distance;
        }
    }

    
    curve
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
