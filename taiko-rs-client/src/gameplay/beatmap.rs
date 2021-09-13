use std::{path::Path};

use crate::{Vector2, render::Color};
use taiko_rs_common::types::PlayMode;
use crate::gameplay::{beatmap_structs::*, defs::*};

/// timing bar color
pub const BAR_COLOR:Color = Color::new(0.0, 0.0, 0.0, 1.0);

#[derive(Clone)]
pub struct Beatmap {
    pub hash: String,
    
    // meta info
    pub metadata: BeatmapMeta,
    pub timing_points: Vec<TimingPoint>,

    pub notes: Vec<NoteDef>,
    pub sliders: Vec<SliderDef>,
    pub spinners: Vec<SpinnerDef>,
    pub holds: Vec<HoldDef>,
}
impl Beatmap {
    pub fn load(file_path:String) -> Beatmap {
        let parent_dir = Path::new(&file_path).parent().unwrap();
        let hash = crate::get_file_hash(file_path.clone()).unwrap();

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

        let lines = crate::read_lines(file_path.clone()).expect("Beatmap file not found");
        let mut current_area = BeatmapSection::Version;
        let mut beatmap = Beatmap {
            metadata: BeatmapMeta::new(file_path.clone(), hash.clone()),
            hash,
            notes: Vec::new(),
            sliders: Vec::new(),
            spinners: Vec::new(),
            holds: Vec::new(),
            timing_points: Vec::new(),
        };

        for line_maybe in lines {
            if let Ok(line) = line_maybe {
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
                        match line.split("v").last().unwrap().trim().parse::<u8>() {
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
                        if key == "CircleSize" {beatmap.metadata.cs = val}
                        if key == "OverallDifficulty" {beatmap.metadata.od = val}
                        if key == "ApproachRate" {beatmap.metadata.ar = val}
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
                        let time = split.next().unwrap().parse::<f32>().unwrap();
                        let read_type = split.next().unwrap().parse::<u64>().unwrap_or(0); // see below


                        let hitsound = split.next().unwrap().parse::<u8>();
                        if let Err(e) = &hitsound {
                            println!("error parsing hitsound: {}", e)
                        }
                        
                        let hitsound = hitsound.unwrap_or(0); // 0 = normal, 2 = whistle, 4 = finish, 8 = clap

                        // read type:
                        // abcdefgh
                        // a = note
                        // b = slider
                        // c = new combo
                        // d, e, f = combo color skip count
                        // g = spinner
                        // h = mania hold
                        let new_combo = (read_type & 4) > 0;
                        let color_skip = 
                              if (read_type & 2u64.pow(4)) > 0 {1} else {0} 
                            + if (read_type & 2u64.pow(5)) > 0 {2} else {0} 
                            + if (read_type & 2u64.pow(6)) > 0 {4} else {0};

                        if (read_type & 2) > 0 { // slider
                            let curve_raw = split.next().unwrap();
                            let mut curve = curve_raw.split('|');
                            let slides = split.next().unwrap().parse::<u64>().unwrap();
                            let length = split.next().unwrap().parse::<f32>().unwrap();
                            let edge_sounds = split
                                .next()
                                .unwrap_or("0")
                                .split("|")
                                .map(|s|s.parse::<u8>().unwrap_or(0)).collect();
                            let edge_sets = split
                                .next()
                                .unwrap_or("0:0")
                                .split("|")
                                .map(|s| {
                                    let mut s2 = s.split(':');
                                    [
                                        s2.next().unwrap_or("0").parse::<u8>().unwrap_or(0),
                                        s2.next().unwrap_or("0").parse::<u8>().unwrap_or(0),
                                    ]
                                })
                                .collect();


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

                            beatmap.sliders.push(SliderDef {
                                raw: line.clone(),
                                pos: Vector2::new(x, y),
                                time,
                                curve_type,
                                curve_points,
                                slides,
                                length,
                                hitsound,
                                hitsamples: HitSamples::from_str(split.next()),
                                edge_sounds,
                                edge_sets,
                                new_combo,
                                color_skip
                            });

                        } else if (read_type & 8) > 0 { // spinner
                            // x,y,time,type,hitSound,...
                            // endTime,hitSample
                            let end_time = split.next().unwrap().parse::<f32>().unwrap();
                            // let length = end_time as f64 - time as f64;

                            beatmap.spinners.push(SpinnerDef {
                                pos: Vector2::new(x, y),
                                time,
                                end_time,
                                hitsound,
                                hitsamples: HitSamples::from_str(split.next()),
                                new_combo,
                                color_skip
                            });
                            // let diff_map = map_difficulty_range(beatmap.metadata.od as f64, 3.0, 5.0, 7.5);
                            // let hits_required:u16 = ((length / 1000.0 * diff_map) * 1.65).max(1.0) as u16; // ((this.Length / 1000.0 * this.MapDifficultyRange(od, 3.0, 5.0, 7.5)) * 1.65).max(1.0)
                            // let spinner = Spinner::new(time, end_time, sv, hits_required);
                            // beatmap.notes.lock().push(Box::new(spinner));
                        } else if (read_type & 2u64.pow(7)) > 0 { // mania hold
                            let end_time = split.next().unwrap().split(":").next().unwrap().parse::<f32>().unwrap();
                            beatmap.holds.push(HoldDef {
                                pos: Vector2::new(x, y),
                                time,
                                end_time,
                                hitsound,
                                hitsamples: HitSamples::from_str(split.next()),
                            });
                        } else { // note
                            beatmap.notes.push(NoteDef {
                                pos: Vector2::new(x, y),
                                time,
                                hitsound,
                                hitsamples: HitSamples::from_str(split.next()),
                                new_combo,
                                color_skip
                            });
                        }
                    }

                    BeatmapSection::Colors => {},
                    BeatmapSection::Editor => {},
                }
            }
        }

        // make sure we have the ar set
        beatmap.metadata.do_checks();

        beatmap
    }

    pub fn from_metadata(metadata: &BeatmapMeta) -> Beatmap {
        // load the betmap
        let mut b = Beatmap::load(metadata.file_path.clone());
        // overwrite the loaded meta with the old meta, this maintains calculations etc
        b.metadata = metadata.clone();
        b
    }

    pub fn beat_length_at(&self, time:f32, allow_multiplier:bool) -> f32 {
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

        let mut mult:f32 = 1.0;
        let p = point.unwrap();

        if allow_multiplier && inherited_point.is_some() {
            let ip = inherited_point.unwrap();

            if p.time < ip.time && ip.beat_length < 0.0 {
                mult = (-ip.beat_length as f32).clamp(10.0, 1000.0) / 100.0;
            }
        }

        p.beat_length as f32 * mult
    }
    
    // something is fucked with this, it returns values wayyyyyyyyyyyyyyyyyyyyyy too high
    pub fn slider_velocity_at(&self, time:f32) -> f32 {
        let bl = self.beat_length_at(time, true);
        100.0 * (self.metadata.slider_multiplier * 1.4) * if bl > 0.0 {1000.0 / bl} else {1.0}
    }

    pub fn control_point_at(&self, time:f32) -> TimingPoint {
        // panic as this should be dealt with earlier in the code
        if self.timing_points.len() == 0 {panic!("beatmap has no timing points!")}

        let mut point = self.timing_points[0];
        for tp in self.timing_points.iter() {
            if tp.time <= time {point = *tp}
        }

        point
    }

    pub fn bpm_multiplier_at(&self, time:f32) -> f32 {
        self.control_point_at(time).bpm_multiplier()
    }
}

// contains beatmap info unrelated to notes and timing points, etc
#[derive(Clone, Debug)]
pub struct BeatmapMeta {
    pub file_path: String,
    pub beatmap_hash: String,

    pub beatmap_version: u8,
    pub mode: PlayMode,
    pub artist: String,
    pub title: String,
    pub artist_unicode: String,
    pub title_unicode: String,
    pub creator: String,
    pub version: String,
    pub audio_filename: String,
    pub image_filename: String,
    pub audio_preview: f32,

    pub duration: f32, // time in ms from first note to last note
    /// song duration mins, used for display
    pub mins: u8,
    /// song duration seconds, used for display
    pub secs: u8,

    pub hp: f32,
    pub od: f32,
    pub cs: f32,
    pub ar: f32,
    // pub sr: f64,

    pub slider_multiplier: f32,
    pub slider_tick_rate: f32
}
impl BeatmapMeta {
    pub fn new(file_path:String, beatmap_hash:String) -> BeatmapMeta {
        let unknown = "Unknown".to_owned();

        BeatmapMeta {
            file_path,
            beatmap_hash,
            beatmap_version: 0,
            mode: PlayMode::Standard,
            artist: unknown.clone(),
            title: unknown.clone(),
            artist_unicode: unknown.clone(),
            title_unicode: unknown.clone(),
            creator: unknown.clone(),
            version: unknown.clone(),
            audio_filename: String::new(),
            image_filename: String::new(),
            audio_preview: 0.0,
            hp: -1.0,
            od: -1.0,
            ar: -1.0,
            cs: -1.0,
            slider_multiplier: 1.4,
            slider_tick_rate: 1.0,

            duration: 0.0,
            mins: 0,
            secs: 0,
        }
    }
    pub fn do_checks(&mut self) {
        if self.ar < 0.0 {self.ar = self.od}
    }

    pub fn set_dur(&mut self, duration: f32) {
        self.duration = duration;
        self.mins = (self.duration / 60000.0).floor() as u8;
        self.secs = ((self.duration / 1000.0) % (self.mins as f32 * 60.0)).floor() as u8;
    }

    /// get the title string with the version
    pub fn version_string(&self) -> String {
        format!("{} - {} [{}]", self.artist, self.title, self.version)  
    }

    /// get the difficulty string (od, hp, sr)
    pub fn diff_string(&self) -> String {
        // format!("od: {:.2} hp: {:.2}, {:.2}*, {}:{}", self.od, self.hp, self.sr, self.mins, self.secs)
        format!("od: {:.2} hp: {:.2}, {}:{}", self.od, self.hp, self.mins, self.secs)
    }

    pub fn filter(&self, filter_str: &str) -> bool {
        self.artist.to_ascii_lowercase().contains(filter_str) 
        || self.artist_unicode.to_ascii_lowercase().contains(filter_str) 
        || self.title.to_ascii_lowercase().contains(filter_str) 
        || self.title_unicode.to_ascii_lowercase().contains(filter_str) 
        || self.creator.to_ascii_lowercase().contains(filter_str) 
        || self.version.to_ascii_lowercase().contains(filter_str) 
    }

    pub fn check_mode_override(&self, override_mode:PlayMode) -> PlayMode {
        if self.mode == PlayMode::Standard {
            override_mode
        } else {
            self.mode
        }
    }
}


// might use this later idk
// pub trait IntoSets {
//     fn sort_into_sets(&self) -> Vec<Vec<BeatmapMeta>>;
// }
// impl IntoSets for Vec<BeatmapMeta> {
//     fn sort_into_sets(&self) -> Vec<Vec<BeatmapMeta>> {
//         todo!()
//     }
// }


// stolen from peppy, /shrug
pub fn map_difficulty(diff:f32, min:f32, mid:f32, max:f32) -> f32 {
    if diff > 5.0 {
        mid + (max - mid) * (diff - 5.0) / 5.0
    } else if diff < 5.0 {
        mid - (mid - min) * (5.0 - diff) / 5.0
    } else {
        mid
    }
}