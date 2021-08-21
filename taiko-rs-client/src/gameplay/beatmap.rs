use std::{path::Path, sync::Arc};

use parking_lot::Mutex;
use taiko_rs_common::types::PlayMode;
use crate::{Vector2, render::Color, gameplay::beatmap_structs::*};

/// timing bar color
pub const BAR_COLOR:Color = Color::new(0.0, 0.0, 0.0, 1.0);

#[derive(Clone)]
pub struct Beatmap {
    pub hash: String,
    
    // meta info
    pub metadata: BeatmapMeta,

    pub timing_points: Vec<TimingPoint>,
    end_time: f64,

    pub notes: Vec<NoteDef>,
    pub sliders: Vec<SliderDef>,
    pub spinners: Vec<SpinnerDef>,
    pub holds: Vec<HoldDef>,
}
impl Beatmap {
    pub fn load(dir:String) -> Arc<Mutex<Beatmap>> {
        let lines = crate::read_lines(dir.clone()).expect("Beatmap file not found");
        let mut body = String::new();
        let mut current_area = BeatmapSection::Version;
        let mut beatmap = Beatmap {
            hash: String::new(),
            notes: Vec::new(),
            sliders: Vec::new(),
            spinners: Vec::new(),
            holds: Vec::new(),
            timing_points: Vec::new(),
            metadata: BeatmapMeta::new(),
            end_time: 0.0,
        };

        let parent_dir = Path::new(&dir).parent().unwrap();

        for line_maybe in lines {
            if let Ok(line) = line_maybe {
                body += &format!("{}\n", line);

                // ignore empty lines
                if line.len() < 2 {continue}

                // check for section change
                if line.starts_with("[") {
                    // this one isnt really necessary
                    if line == "[General]" {current_area = BeatmapSection::General}
                    if line == "[Editor]" {current_area = BeatmapSection::Editor}
                    if line == "[Metadata]" {current_area = BeatmapSection::Metadata}
                    if line == "[Difficulty]" {current_area = BeatmapSection::Difficulty}
                    if line == "[Events]" {current_area = BeatmapSection::Events}
                    if line == "[Colours]" {current_area = BeatmapSection::Colors}
                    if line == "[TimingPoints]" {current_area = BeatmapSection::TimingPoints}
                    if line == "[HitObjects]" {
                        // sort timing points before moving onto hitobjects
                        beatmap.timing_points.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());

                        current_area = BeatmapSection::HitObjects; 
                    }
                    continue;
                }

                // not a change in area, check line
                match current_area {
                    BeatmapSection::Version => {
                        match line.split("v").last().unwrap().trim().parse::<f32>() {
                            Ok(v) => beatmap.metadata.beatmap_version = v,
                            Err(e) => println!("error parsing beatmap version: {}", e),
                        }
                    }
                    BeatmapSection::General => {
                        let mut split = line.split(":");
                        let key = split.next().unwrap().trim();
                        let val = split.next().unwrap().trim();

                        if key == "AudioFilename" {beatmap.metadata.audio_filename = parent_dir.join(val).to_str().unwrap().to_owned()}
                        if key == "PreviewTime" {beatmap.metadata.audio_preview = val.parse().unwrap_or(0.0)}
                        if key == "Mode" {
                            let m = val.parse::<u8>().unwrap();
                            beatmap.metadata.mode = m.into();
                        }
                    }
                    BeatmapSection::Metadata => {
                        let mut split = line.split(":");
                        let key = split.next().unwrap().trim();
                        let val = split.next().unwrap().trim();
                        
                        if key == "Title" {beatmap.metadata.title = val.to_owned()}
                        if key == "TitleUnicode" {beatmap.metadata.title_unicode = val.to_owned()}
                        if key == "Artist" {beatmap.metadata.artist = val.to_owned()}
                        if key == "ArtistUnicode" {beatmap.metadata.artist_unicode = val.to_owned()}
                        if key == "Creator" {beatmap.metadata.creator = val.to_owned()}
                        if key == "Version" {beatmap.metadata.version = val.to_owned()}
                    }
                    BeatmapSection::Difficulty => {
                        let mut split = line.split(":");
                        let key = split.next().unwrap().trim();
                        let val = split.next().unwrap().trim().parse::<f32>().unwrap();

                        if key == "HPDrainRate" {beatmap.metadata.hp = val}
                        if key == "OverallDifficulty" {beatmap.metadata.od = val}
                        if key == "SliderMultiplier" {beatmap.metadata.slider_multiplier = val}
                        if key == "SliderTickRate" {beatmap.metadata.slider_tick_rate = val}
                    }
                    BeatmapSection::Events => {
                        let mut split = line.split(',');
                        // eventType,startTime,eventParams
                        // 0,0,filename,xOffset,yOffset
                        let event_type = split.next().unwrap();

                        if event_type == "0" && split.next().unwrap() == "0" {
                            let filename = split.next().unwrap().to_owned();
                            let filename = filename.trim_matches('"');
                            beatmap.metadata.image_filename = parent_dir.join(filename).to_str().unwrap().to_owned();
                        
                        }
                    }
                    BeatmapSection::TimingPoints => {
                        let tp = TimingPoint::from_str(&line);

                        beatmap.timing_points.push(tp);
                    }
                    BeatmapSection::HitObjects => {
                        let mut split = line.split(",");
                        if split.clone().count() < 2 {continue} // skip empty lines

                        let x = split.next().unwrap().parse::<f64>().unwrap();
                        let y = split.next().unwrap().parse::<f64>().unwrap();
                        let time = split.next().unwrap().parse::<f64>().unwrap();
                        let read_type = split.next().unwrap().parse::<u64>().unwrap_or(0); // note, slider, spinner, hold
                        let hitsound = split.next().unwrap().parse::<u32>().unwrap_or(0); // 0 = normal, 2 = whistle, 4 = finish, 8 = clap

                        if (read_type & 2) > 0 { // slider
                            let curve_raw = split.next().unwrap();
                            let mut curve = curve_raw.split('|');
                            let slides = split.next().unwrap().parse::<u64>().unwrap();
                            let length = split.next().unwrap().parse::<f64>().unwrap();
                            let edge_sounds = split
                                .next()
                                .unwrap_or("")
                                .split("|")
                                .map(|s|s.parse::<u8>().unwrap_or(0)).collect();
                            let edge_sets = split
                                .next()
                                .unwrap_or("0")
                                .split("|")
                                .map(|s|s.parse::<u8>().unwrap_or(0)).collect();


                            let curve_type = match &*curve.next().unwrap() {
                                "B" => CurveType::Bézier,
                                "P" => CurveType::Perfect,
                                "C" => CurveType::Catmull,
                                "L" => CurveType::Linear,
                                _ => CurveType::Linear
                            };

                            let mut curve_points = Vec::new();
                            while let Some(pair) = curve.next() {
                                let mut s = pair.split(':');
                                curve_points.push(Vector2::new(
                                    s.next().unwrap().parse().unwrap(),
                                    s.next().unwrap().parse().unwrap()
                                ))
                            }

                            if beatmap.metadata.mode == PlayMode::Catch {
                                println!("curve: {}, points: {:?}", curve_raw, curve_points);
                            }

                            beatmap.sliders.push(SliderDef {
                                pos: Vector2::new(x, y),
                                time,
                                curve_type,
                                curve_points,
                                slides,
                                length,
                                hitsound,
                                hitsamples: Vec::new(),
                                edge_sounds,
                                edge_sets
                            });

                        } else if (read_type & 8) > 0 { // spinner
                            // x,y,time,type,hitSound,...
                            // endTime,hitSample
                            let end_time = split.next().unwrap().parse::<f64>().unwrap();
                            // let length = end_time as f64 - time as f64;

                            beatmap.spinners.push(SpinnerDef {
                                pos: Vector2::new(x, y),
                                time,
                                end_time,
                                hitsound,
                                hitsamples: Vec::new()
                            });
                            // let diff_map = map_difficulty_range(beatmap.metadata.od as f64, 3.0, 5.0, 7.5);
                            // let hits_required:u16 = ((length / 1000.0 * diff_map) * 1.65).max(1.0) as u16; // ((this.Length / 1000.0 * this.MapDifficultyRange(od, 3.0, 5.0, 7.5)) * 1.65).max(1.0)
                            // let spinner = Spinner::new(time, end_time, sv, hits_required);
                            // beatmap.notes.lock().push(Box::new(spinner));
                        } else if (read_type & 2u64.pow(7)) > 0 { // mania hold
                            let end_time = split.next().unwrap().split(":").next().unwrap().parse::<f64>().unwrap();
                            beatmap.holds.push(HoldDef {
                                pos: Vector2::new(x, y),
                                time,
                                end_time,
                                hitsound,
                                hitsamples: Vec::new()
                            });

                        } else { // note

                            beatmap.notes.push(NoteDef {
                                pos: Vector2::new(x, y),
                                time,
                                hitsound,
                                hitsamples: Vec::new()
                            });
                            // let note = Note::new(time, hit_type, finisher, sv);
                            // beatmap.notes.lock().push(Box::new(note));
                        }
                    }

                    // dont need to do anything with these for the scope of this game
                    BeatmapSection::Editor => {},
                    BeatmapSection::Colors => {},
                }
            }
        }

        // beatmap.notes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        // let start_time = beatmap.notes.first().unwrap().time() as f64;
        // let end_time = beatmap.notes.last().unwrap().end_time(0.0) as f64;

        beatmap.hash = format!("{:x}", md5::compute(body).to_owned());
        // beatmap.metadata.set_dur((end_time - start_time) as u64);
        // beatmap.end_time = end_time;

        // this might be an issue later on *maybe*
        // beatmap.calc_sr();
        Arc::new(Mutex::new(beatmap))
    }

    // pub fn calc_sr(&mut self) {self.metadata.sr = DifficultyCalculator::new(Arc::new(Mutex::new(self.to_owned()))).compute_difficulty()}

    pub fn beat_length_at(&self, time:f64, allow_multiplier:bool) -> f64 {
        if self.timing_points.len() == 0 {return 0.0}

        let mut point: Option<TimingPoint> = Some(self.timing_points.as_slice()[0].clone());
        let mut inherited_point: Option<TimingPoint> = None;

        for tp in self.timing_points.as_slice() {
            if tp.time <= time {
                if tp.is_inherited() {
                    inherited_point = Some(tp.clone());
                } else {
                    point = Some(tp.clone());
                }
            }
        }

        let mut mult:f64 = 1.0;
        let p = point.unwrap();

        if allow_multiplier && inherited_point.is_some() {
            let ip = inherited_point.unwrap();

            if p.time < ip.time && ip.beat_length < 0.0 {
                mult = (-ip.beat_length as f64).clamp(10.0, 1000.0) / 100.0;
            }
        }

        p.beat_length as f64 * mult
    }
    // something is fucked with this, it returns values wayyyyyyyyyyyyyyyyyyyyyy too high
    pub fn slider_velocity_at(&self, time:u64) -> f64 {
        let bl = self.beat_length_at(time as f64, true);
        100.0 * (self.metadata.slider_multiplier as f64 * 1.4) * if bl > 0.0 {1000.0 / bl} else {1.0}
    }

    pub fn control_point_at(&self, time:f64) -> TimingPoint {
        // panic as this should be dealt with earlier in the code
        if self.timing_points.len() == 0 {panic!("beatmap has no timing points!")}

        let mut point = self.timing_points[0];
        for tp in self.timing_points.iter() {
            if tp.time <= time {point = *tp}
        }

        point
    }

    pub fn bpm_multiplier_at(&self, time:f64) -> f64 {
        self.control_point_at(time).bpm_multiplier()
    }
}

#[derive(Clone, Debug)]
pub struct NoteDef {
    pub pos: Vector2,
    pub time: f64,
    pub hitsound: u32,
    pub hitsamples: Vec<u8>
}

#[derive(Clone, Debug)]
pub struct SliderDef {
    // x,y,time,type,hitSound,curveType|curvePoints,slides,length,edgeSounds,edgeSets,hitSample
    pub pos: Vector2,
    pub time: f64,
    pub hitsound: u32,
    pub curve_type: CurveType,
    pub curve_points: Vec<Vector2>,
    pub slides: u64,
    pub length: f64,
    pub edge_sounds: Vec<u8>,
    pub edge_sets: Vec<u8>,
    
    pub hitsamples: Vec<u8>
}

#[derive(Clone, Debug)]
pub struct SpinnerDef {
    pub pos: Vector2,
    pub time: f64,
    pub hitsound: u32,
    pub end_time: f64,
    
    pub hitsamples: Vec<u8>
}

#[derive(Clone, Debug)]
pub struct HoldDef {
    pub pos: Vector2,
    pub time: f64,
    pub hitsound: u32,
    pub end_time: f64,
    
    pub hitsamples: Vec<u8>
}



#[derive(Clone, Copy, Debug)]
pub enum CurveType {
    Bézier,
    Catmull,
    Linear,
    Perfect
}

// contains beatmap info unrelated to notes and timing points, etc
#[derive(Clone, Debug)]
pub struct BeatmapMeta {
    pub mode: PlayMode,
    pub beatmap_version: f32,
    pub artist: String,
    pub title: String,
    pub artist_unicode: String,
    pub title_unicode: String,
    pub creator: String,
    pub version: String,
    pub audio_filename: String,
    pub image_filename: String,
    pub audio_preview: f32,

    pub duration: u64, // time in ms from first note to last note
    mins: u8,
    secs: u8,

    pub hp: f32,
    pub od: f32,
    pub sr: f64,
    pub slider_multiplier: f32,
    pub slider_tick_rate: f32,
}
impl BeatmapMeta {
    fn new() -> BeatmapMeta {
        let unknown = "Unknown".to_owned();

        BeatmapMeta {
            mode: PlayMode::Taiko,
            beatmap_version: 0.0,
            artist: unknown.clone(),
            title: unknown.clone(),
            artist_unicode: unknown.clone(),
            title_unicode: unknown.clone(),
            creator: unknown.clone(),
            version: unknown.clone(),
            audio_filename: String::new(),
            image_filename: String::new(),
            audio_preview: 0.0,
            hp: 0.0,
            od: 0.0,
            sr: 0.0,
            slider_multiplier: 1.4,
            slider_tick_rate: 1.0,

            duration: 0,
            mins: 0,
            secs: 0,
        }
    }
    pub fn set_dur(&mut self, duration:u64) {
        self.duration = duration;
        self.mins = (self.duration as f32 / 60000.0).floor() as u8;
        self.secs = ((self.duration as f32 / 1000.0) % (self.mins as f32 * 60.0)).floor() as u8;
    }

    /// get the title string with the version
    pub fn version_string(&self) -> String {
        format!("{} - {} [{}]", self.artist, self.title, self.version)  
    }

    /// get the difficulty string (od, hp, sr)
    pub fn diff_string(&self) -> String {
        format!("od: {:.2} hp: {:.2}, {:.2}*, {}:{}", self.od, self.hp, self.sr, self.mins, self.secs)
    }
}


/// helper enum
#[derive(Debug)]
enum BeatmapSection {
    Version,
    General,
    Editor,
    Metadata,
    Difficulty,
    Events,
    TimingPoints,
    Colors,
    HitObjects,
}


// stolen from peppy, /shrug
pub fn map_difficulty_range(diff:f64, min:f64, mid:f64, max:f64) -> f64 {
    if diff > 5.0 {
        mid + (max - mid) * (diff - 5.0) / 5.0
    } else if diff < 5.0 {
        mid - (mid - min) * (5.0 - diff) / 5.0
    } else {
        mid
    }
}