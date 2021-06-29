use std::{path::Path, sync::{Arc, Mutex}, time::SystemTime};

use cgmath::Vector2;
use piston::RenderArgs;

use super::{*, diff_calc::DifficultyCalculator, beatmap_structs::*};
use crate::{HIT_AREA_RADIUS, HIT_POSITION, PLAYFIELD_RADIUS, WINDOW_SIZE, game::{Audio, Settings}};
use crate::{NOTE_RADIUS, enums::Playmode, game::{SoundEffect, get_font}, render::{Renderable, Rectangle, Text, Circle, Color, Border}};

const LEAD_IN_TIME:f32 = 1000.0; // how much time should pass at beatmap start before audio begins playing (and the map "starts")
const BAR_WIDTH:f64 = 4.0; // how wide is a timing bar
const BAR_COLOR:[f32;4] = [0.0,0.0,0.0,1.0]; // timing bar color
const BAR_SPACING:f64 = 4.0; // how many beats between timing bars
const SV_FACTOR:f64 = 700.0; // bc sv is bonked, divide it by this amount
const DURATION_HEIGHT:f64 = 35.0; // how tall is the duration bar
const OFFSET_DRAW_TIME:i64 = 2_000;

#[derive(Clone)]
pub struct Beatmap {
    pub score: Score,

    pub hash: String,
    pub started: bool,
    pub completed: bool,
    
    // lists
    pub notes: Vec<Arc<Mutex<dyn HitObject>>>,
    timing_bars: Vec<Arc<Mutex<TimingBar>>>,
    hit_timings: Vec<(i64, i64)>, // map time, diff (note - hit) //TODO: figure out how to draw this efficiently

    note_index: usize,
    pub timing_points: Vec<TimingPoint>,
    timing_point_index: usize,

    pub song: SoundEffect,
    pub song_start: SystemTime,
    lead_in_time: f32,
    end_time: f64,

    // offset things
    offset: i64,
    offset_changed_time: i64,

    // meta info
    pub metadata: BeatmapMeta
}
impl Beatmap {
    pub fn load(dir:String) -> Arc<Mutex<Beatmap>> {
        let lines = crate::read_lines(dir.clone()).expect("Beatmap file not found");
        let mut body = String::new();
        let mut current_area = BeatmapSection::Version;
        let mut meta = BeatmapMeta::new();
        let beatmap = Arc::new(Mutex::new(Beatmap {
            hash: String::new(),
            notes: Vec::new(),
            timing_points: Vec::new(),
            timing_bars: Vec::new(),
            hit_timings: Vec::new(),
            song_start: SystemTime::now(),
            score: Score::new(String::new()),
            metadata: BeatmapMeta::new(),
            song: SoundEffect::new_empty(), // temp until we get the audio file path
            note_index: 0,
            timing_point_index: 0,
            started: false,
            completed: false,
            end_time: 0.0,
            lead_in_time: LEAD_IN_TIME,

            offset: 0,
            offset_changed_time: 0
        }));

        let parent_dir = Path::new(&dir).parent().unwrap();
        let mut tp_parent: Option<Arc<TimingPoint>> = None;

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
                        let mut b2 = beatmap.lock().unwrap();
                        b2.timing_points.sort_by(|a, b| a.time.cmp(&b.time));

                        current_area = BeatmapSection::HitObjects; 
                    }
                    continue;
                }

                // not a change in area, check line
                match current_area {
                    BeatmapSection::Version => {
                        let v = line.split("v").last().unwrap().trim().parse::<f32>();
                        if let Ok(v) = v {
                            meta.beatmap_version = v;
                        } else if let Err(e) = v {
                            println!("error parsing beatmap version: {}", e);
                        }
                    },
                    BeatmapSection::General => {
                        let mut split = line.split(":");
                        let key = split.next().unwrap().trim();
                        let val = split.next().unwrap().trim();

                        if key == "AudioFilename" {meta.audio_filename = parent_dir.join(val).to_str().unwrap().to_owned();}
                        if key == "Mode" {
                            let m = val.parse::<u8>().unwrap();
                            meta.mode = m.into();
                        }
                    },
                    BeatmapSection::Metadata => {
                        let mut split = line.split(":");
                        let key = split.next().unwrap().trim();
                        let val = split.next().unwrap().trim();
                        
                        if key == "Title" {meta.title = val.to_owned()}
                        if key == "TitleUnicode" {meta.title_unicode = val.to_owned()}
                        if key == "Artist" {meta.artist = val.to_owned()}
                        if key == "ArtistUnicode" {meta.artist_unicode = val.to_owned()}
                        if key == "Creator" {meta.creator = val.to_owned()}
                        if key == "Version" {meta.version = val.to_owned()}
                    },
                    BeatmapSection::Difficulty => {
                        let mut split = line.split(":");
                        let key = split.next().unwrap().trim();
                        let val = split.next().unwrap().trim().parse::<f32>().unwrap();

                        if key == "HPDrainRate" {meta.hp = val}
                        if key == "OverallDifficulty" {meta.od = val}
                        if key == "SliderMultiplier" {meta.slider_multiplier = val}
                        if key == "SliderTickRate" {meta.slider_tick_rate = val}
                    },
                    BeatmapSection::Events => {
                        let mut split = line.split(',');
                        // eventType,startTime,eventParams
                        // 0,0,filename,xOffset,yOffset
                        let event_type = split.next().unwrap();

                        if event_type == "0" {
                            if split.next().unwrap() == "0" {
                                let filename = split.next().unwrap().to_owned();
                                let filename = filename.trim_matches('"');
                                meta.image_filename = parent_dir.join(filename).to_str().unwrap().to_owned();
                            }
                        }
                    },
                    BeatmapSection::TimingPoints => {
                        let tp = TimingPoint::from_str(&line, tp_parent.clone());

                        if !tp.is_inherited() {
                            tp_parent = Some(Arc::new(tp.clone()));
                        }

                        beatmap.lock().unwrap().timing_points.push(tp);
                    },
                    BeatmapSection::HitObjects => {
                        let mut split = line.split(",");
                        if split.clone().count() < 2 {continue;} // skip empty lines

                        let _x = split.next();
                        let _y = split.next();
                        let time = split.next().unwrap().parse::<u64>().unwrap();
                        let read_type = split.next().unwrap().parse::<u64>().unwrap(); // note, slider, spinner
                        let hitsound = split.next().unwrap().parse::<u32>().unwrap(); // 0 = normal, 2 = whistle, 4 = finish, 8 = clap

                        let hit_type = if (hitsound & (2 | 8)) > 0 {super::HitType::Kat} else {super::HitType::Don};
                        let finisher = (hitsound & 4) > 0;
                        
                        // set later, bc for some reason its inconsistant here
                        let sv = 1.0; //beatmap.lock().unwrap().slider_velocity_at(time) / SV_FACTOR;

                        if (read_type & 2) > 0 { // slider
                            let _curve = split.next(); // dont care
                            let slides = split.next().unwrap().parse::<u64>().unwrap();
                            let length = split.next().unwrap().parse::<f64>().unwrap();

                            let l = (length * 1.4) * slides as f64;
                            let v2 = 100.0 * (meta.slider_multiplier as f64 * 1.4);
                            let bl = beatmap.lock().unwrap().beat_length_at(time as f64, true);
                            let end_time = time + (l / v2 * bl) as u64;
                            
                            // convert vars
                            let v = beatmap.lock().unwrap().slider_velocity_at(time);
                            let bl = beatmap.lock().unwrap().beat_length_at(time as f64, meta.beatmap_version < 8.0);
                            let skip_period = (bl / meta.slider_tick_rate as f64).min((end_time - time) as f64 / slides as f64);

                            if skip_period > 0.0 && meta.mode != Playmode::Taiko && l / v * 1000.0 < 2.0 * bl {
                                let mut i = 0;
                                let mut j = time as f64;

                                // load sounds
                                let sound_list_raw = if let Some(list) = split.next() {list.split("|")} else {"".split("")};

                                // when loading, if unified just have it as sound_types with 1 index
                                let mut sound_types:Vec<(HitType, bool)> = Vec::new();

                                for i in sound_list_raw {
                                    if let Ok(hitsound) = i.parse::<u32>() {
                                        let hit_type = if (hitsound & (2 | 8)) > 0 {super::HitType::Kat} else {super::HitType::Don};
                                        let finisher = (hitsound & 4) > 0;
                                        sound_types.push((hit_type, finisher));
                                    }
                                }
                                
                                let unified_sound_addition = sound_types.len() == 0;
                                if unified_sound_addition {
                                    sound_types.push((HitType::Don, false));
                                }

                                loop {
                                    let sound_type = sound_types[i];

                                    let note = Note::new(
                                        beatmap.clone(),
                                        j as u64,
                                        sound_type.0,
                                        sound_type.1,
                                        sv
                                    );
                                    beatmap.lock().unwrap().notes.push(Arc::new(Mutex::new(note)));

                                    if !unified_sound_addition {
                                        i = (i + 1) % sound_types.len();
                                    }

                                    j += skip_period;
                                    if !(j < end_time as f64 + skip_period / 8.0) {break}
                                }
                            } else {
                                let slider = Slider::new(beatmap.clone(), time, end_time, finisher, sv);
                                beatmap.lock().unwrap().notes.push(Arc::new(Mutex::new(slider)));
                            }

                        } else if (read_type & 8) > 0 { // spinner
                            // x,y,time,type,hitSound,...
                            // endTime,hitSample
                            let end_time = split.next().unwrap().parse::<u64>().unwrap();
                            let length = end_time as f64 - time as f64;

                            let diff_map:f64;
                            {
                                let diff = meta.od as f64;
                                let min = 3.0;
                                let mid = 5.0;
                                let max = 7.5;
                                if diff > 5.0 {
                                    diff_map = mid + (max - mid) * (diff - 5.0) / 5.0;
                                } else if diff < 5.0 {
                                    diff_map = mid - (mid - min) * (5.0 - diff) / 5.0;
                                } else {
                                    diff_map = mid;
                                }
                            }

                            let hits_required:u16 = ((length / 1000.0 * diff_map) * 1.65).max(1.0) as u16; // ((this.Length / 1000.0 * this.MapDifficultyRange(od, 3.0, 5.0, 7.5)) * 1.65).max(1.0)
                            // just make a slider for now
                            let spinner = Spinner::new(beatmap.clone(), time, end_time, sv, hits_required);
                            beatmap.lock().unwrap().notes.push(Arc::new(Mutex::new(spinner)));
                        } else { // note
                            let note = Note::new(beatmap.clone(), time, hit_type, finisher, sv);
                            beatmap.lock().unwrap().notes.push(Arc::new(Mutex::new(note)));
                        }
                    },

                    // dont need to do anything with these for the scope of this game
                    BeatmapSection::Editor => {},
                    BeatmapSection::Colors => {},
                }
            }
        }

        
        let md5 = format!("{:x}", md5::compute(body).to_owned());
        // does this need to be in its own scope? probably not but whatever
        {
            let mut locked = beatmap.lock().unwrap();
            let start_time = locked.clone().notes.first().unwrap().lock().unwrap().time() as f64;
            let end_time = locked.clone().notes.last().unwrap().lock().unwrap().end_time() as f64;

            meta.set_dur((end_time - start_time) as u64);
            locked.end_time = end_time;
            locked.metadata = meta.clone();
            locked.calc_sr();
            locked.song = SoundEffect::new(&meta.clone().audio_filename);

            locked.hash = md5.clone();
            locked.score = Score::new(md5.clone());
        }
        beatmap
    }

    pub fn calc_sr(&mut self) {
        let mut calc = DifficultyCalculator::new(Arc::new(Mutex::new(self.to_owned())));
        self.metadata.sr = calc.compute_difficulty();
    }

    pub fn time(&self) -> i64 {
        self.song.duration() as i64 - (self.lead_in_time as i64 + 50) - self.offset
    }
    pub fn increment_offset(&mut self, delta:i64) {
        self.offset += delta;
        self.offset_changed_time = self.time();
        println!("offset is now {}", self.offset);
    }
    pub fn next_note(&mut self) {
        self.note_index += 1;
    }

    pub fn hit(&mut self, hit_type:HitType) {
        let mut sound = match hit_type {HitType::Don => "don",HitType::Kat => "kat"};

        // if theres no more notes to hit, return
        if self.note_index >= self.notes.len() {
            Audio::play(sound);
            return;
        }
        let time = self.time() as f64;

        // check for finisher 2nd hit. 
        if self.note_index > 0 {
            let mut last_note = self.notes[self.note_index-1].lock().unwrap();

            match last_note.check_finisher(hit_type, time) {
                ScoreHit::Miss => {return},
                ScoreHit::X100 => {
                    self.score.add_pts(100, true);
                    return;
                },
                ScoreHit::X300 => {
                    self.score.add_pts(300, true);
                    return;
                },
                ScoreHit::Other(points, _) => {
                    self.score.add_pts(points as u64, false);
                    return;
                },
                ScoreHit::None => {},
            }
        }

        let note = self.notes[self.note_index].clone();
        let mut note = note.lock().unwrap();
        let note_time = note.time() as f64;

        match note.get_points(hit_type, time) {
            ScoreHit::None => {
                // play sound
                Audio::play(sound);
            },
            ScoreHit::Miss => {
                self.score.hit_miss(time as u64, note_time as u64);
                self.next_note();
                Audio::play(sound);
                //TODO: play miss sound
                //TODO: indicate this was a miss
            },
            ScoreHit::X100 => {
                self.score.hit100(time as u64, note_time as u64);

                // only play finisher sounds if the note is both a finisher and was hit
                // could maybe also just change this to HitObject.get_sound() -> &str
                if note.finisher_sound() {sound = match hit_type {HitType::Don => "bigdon",HitType::Kat => "bigkat"};}
                Audio::play(sound);
                //TODO: indicate this was a bad hit


                self.next_note();
            },
            ScoreHit::X300 => {
                self.score.hit300(time as u64, note_time as u64);
                
                if note.finisher_sound() {sound = match hit_type {HitType::Don => "bigdon",HitType::Kat => "bigkat"};}
                Audio::play(sound);


                self.next_note();
            },
            ScoreHit::Other(score, consume) => { // used by sliders and spinners
                self.score.score += score as u64;
                if consume {self.next_note();}

                Audio::play(sound);
            }
        }
    }

    pub fn update(&mut self) {
        if self.lead_in_time > 0.0 {
            let elapsed = self.song_start.elapsed().unwrap().as_micros() as f32 / 1000.0;
            self.song_start = SystemTime::now();
            self.lead_in_time -= elapsed;

            if self.lead_in_time <= 0.0 {
                self.lead_in_time = 0.0;
                self.song.play();
            }
        }

        let time = self.time();

        for note in self.notes.iter_mut() {
            note.lock().unwrap().update(time);
        }

        // if theres no more notes to hit, show score screen
        if self.note_index >= self.notes.len() {
            self.completed = true;
            self.timing_bars.clear();
            return;
        }

        if (self.notes[self.note_index].lock().unwrap().end_time() as i64) < time {
            if self.notes[self.note_index].lock().unwrap().causes_miss() {
                // need to set these manually instead of score.hit_miss,
                // since we dont want to add anything to the hit error list
                self.score.xmiss += 1;
                self.score.combo = 0;
            }

            self.next_note();
        }
        
        for tb in self.timing_bars.iter_mut() {
            tb.lock().unwrap().update(time as f64);
        }

        // check timing point
        if self.timing_point_index + 1 < self.timing_points.len() && self.timing_points[self.timing_point_index + 1].time as i64 <= time {
            self.timing_point_index += 1;
        }
    }
    pub fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut renderables: Vec<Box<dyn Renderable>> = Vec::new();
        // load this here, it a bit more performant
        let font = get_font("main");
        let time = self.time();

        // draw the playfield
        let playfield = Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            f64::MAX-4.0,
            Vector2::new(0.0, HIT_POSITION.y - (PLAYFIELD_RADIUS + 2.0)),
            Vector2::new(args.window_size[0], (PLAYFIELD_RADIUS+2.0) * 2.0),
            if self.timing_points[self.timing_point_index].kiai {
                Some(Border::new(Color::YELLOW, 2.0))
            } else {None}
        );
        renderables.push(Box::new(playfield));

        // draw the hit area
        let hit_area = Circle::new(
            Color::BLACK,
            f64::MAX,
            HIT_POSITION,
            HIT_AREA_RADIUS + 2.0
        );
        renderables.push(Box::new(hit_area));

        // score text
        let score_text = Text::new(
            Color::BLACK,
            0.0,
            Vector2::new(args.window_size[0] - 200.0, 40.0),
            30,
            crate::format(self.score.score),
            font.clone()
        );
        renderables.push(Box::new(score_text));

        // acc text
        let acc_text = Text::new(
            Color::BLACK,
            0.0,
            Vector2::new(args.window_size[0] - 200.0, 70.0),
            30,
            format!("{:.2}%", self.score.acc()*100.0),
            font.clone()
        );
        renderables.push(Box::new(acc_text));

        // combo text
        let mut combo_text = Text::new(
            Color::WHITE,
            0.0,
            HIT_POSITION - Vector2::new(100.0, 0.0),
            30,
            crate::format(self.score.combo),
            font.clone()
        );
        combo_text.center_text(Rectangle::bounds_only(
            Vector2::new(0.0, HIT_POSITION.y - HIT_AREA_RADIUS/2.0),
            Vector2::new(HIT_POSITION.x - NOTE_RADIUS, HIT_AREA_RADIUS)
        ));
        renderables.push(Box::new(combo_text));

        // draw offset
        if self.offset_changed_time > 0 && time - self.offset_changed_time < OFFSET_DRAW_TIME {
            let mut offset_text = Text::new(
                Color::BLACK,
                -20.0,
                Vector2::new(0.0,0.0), // centered anyways
                32,
                format!("Offset: {}", self.offset),
                font.clone()
            );
            offset_text.center_text(Rectangle::bounds_only(Vector2::new(0.0,0.0), Vector2::new(WINDOW_SIZE.x as f64, HIT_POSITION.y)));
            renderables.push(Box::new(offset_text));
        }

        // duration bar
        // duration remaining
        let duration_border = Rectangle::new(
            Color::TRANSPARENT_WHITE,
            1.0,
            Vector2::new(0.0, args.window_size[1] - (DURATION_HEIGHT + 3.0)),
            Vector2::new(args.window_size[0], DURATION_HEIGHT),
            Some(Border::new(Color::BLACK, 1.8))
        );
        renderables.push(Box::new(duration_border));
        // fill
        let duration_fill = Rectangle::new(
            [0.4,0.4,0.4,1.0].into(),
            2.0,
            Vector2::new(0.0, args.window_size[1] - (DURATION_HEIGHT + 3.0)),
            Vector2::new(args.window_size[0] * (self.time() as f64/self.end_time), DURATION_HEIGHT),
            None
        );
        renderables.push(Box::new(duration_fill));


        // draw hit timings bar

        // draw hit timings

        // draw notes
        for note in self.notes.iter_mut() {renderables.extend(note.lock().unwrap().draw(args));}
        // draw timing lines
        for tb in self.timing_bars.iter_mut() {renderables.extend(tb.lock().unwrap().draw(args))}

        renderables
    }

    pub fn start(&mut self) {
        if !self.started {
            self.song.stop();
            self.song_start = SystemTime::now(); //TODO: remove this actually, time() should be based off the song duration
            self.started = true;
            self.lead_in_time = LEAD_IN_TIME;
            return;
        }
        self.song.play();
    }
    pub fn pause(&mut self) {
        self.song.pause();
        // is there anything else we need to do?

        // might mess with lead-in but meh
    }
    pub fn reset(&mut self) {
        let settings = Settings::get().clone();

        let c = self.clone();
        for note in self.notes.as_mut_slice() {
            let mut note = note.lock().unwrap();

            // set note svs
            if settings.static_sv {
                note.set_sv(settings.sv_multiplier as f64);
            } else {
                let sv = c.slider_velocity_at(note.time()) / SV_FACTOR;
                note.set_sv(sv);
            }

            note.reset();
            note.set_od(self.metadata.od as f64); //TODO! change when adding mods
        }
        self.note_index = 0;
        self.song.stop();
        self.completed = false;
        self.started = false;
        self.lead_in_time = LEAD_IN_TIME;

        // setup timing bars
        //TODO: it would be cool if we didnt actually need timing bar objects, and could just draw them
        if self.timing_bars.len() == 0 {
            // load timing bars
            let end_time = self.end_time + 500.0 * self.beat_length_at(self.end_time, false);
            let mut time = self.timing_points[0].time as f64;
            let mut sv = settings.sv_multiplier as f64;

            // TODO: instead of just doing 500, actually get how many are needed before the first timing point lol
            time -= 500.0 * self.beat_length_at(time, false);
            loop {
                if !settings.static_sv {sv = self.slider_velocity_at(time as u64) / SV_FACTOR}

                // add timing bar at current time
                self.timing_bars.push(Arc::new(Mutex::new(TimingBar::new(time as u64, sv))));

                // why isnt this accounting for bpm changes?
                time += self.beat_length_at(time, false) * BAR_SPACING;
                if time >= end_time {break}
            }

            self.timing_bars.remove(0); // not sure if this is still necessary
            println!("created {} timing bars", self.timing_bars.len());
        }
    
        self.score = Score::new(self.hash.clone());
    }

    pub fn beat_length_at(&self, time:f64, allow_multiplier:bool) -> f64 {
        if self.timing_points.len() == 0 {return 0.0}

        let mut point: Option<TimingPoint> = Some(self.timing_points.as_slice()[0].clone());
        let mut inherited_point: Option<TimingPoint> = None;

        for tp in self.timing_points.as_slice() {
            if (tp.time as f64) <= time {
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

        p.beat_length as f64 * mult as f64
    }
    pub fn slider_velocity_at(&self, time:u64) -> f64 {
        let bl = self.beat_length_at(time as f64, true);
        if bl > 0.0 {
            return 100.0 * (self.metadata.slider_multiplier as f64 * 1.4) * (1000.0 / bl);
        }

        100.0 * (self.metadata.slider_multiplier as f64 * 1.4)
    }
}


// timing bar struct
struct TimingBar {
    time: u64,
    speed: f64,
    pos: Vector2<f64>
}
impl TimingBar {
    pub fn new(time:u64, speed:f64) -> TimingBar {
        TimingBar {
            time, 
            speed,
            pos: Vector2::new(0.0, 0.0)
        }
    }

    pub fn update(&mut self, time:f64) {
        self.pos = HIT_POSITION + Vector2::new(((self.time as f64 - time as f64) * self.speed) - BAR_WIDTH / 2.0, -PLAYFIELD_RADIUS);
    }

    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut renderables: Vec<Box<dyn Renderable>> = Vec::new();
        if self.pos.x + BAR_WIDTH < 0.0 || self.pos.x - BAR_WIDTH > args.window_size[0] as f64 {return renderables;}

        renderables.push(Box::new(Rectangle::new(
            BAR_COLOR.into(),
            f64::MAX-5.0,
            self.pos,
            Vector2::new(BAR_WIDTH, PLAYFIELD_RADIUS*2.0),
            None
        )));

        renderables
    }
}


// contains beatmap info unrelated to notes and timing points, etc
#[derive(Clone, Debug)]
pub struct BeatmapMeta {
    pub mode: Playmode,
    pub beatmap_version:f32,
    pub artist: String,
    pub title: String,
    pub artist_unicode: String,
    pub title_unicode: String,
    pub creator: String,
    pub version: String,
    pub audio_filename: String,
    pub image_filename: String,

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
            mode: Playmode::Taiko,
            beatmap_version: 0.0,
            artist: unknown.clone(),
            title: unknown.clone(),
            artist_unicode: unknown.clone(),
            title_unicode: unknown.clone(),
            creator: unknown.clone(),
            version: unknown.clone(),
            audio_filename: String::new(),
            image_filename: String::new(),
            hp: 0.0,
            od: 0.0,
            sr: 0.0,
            slider_multiplier: 1.4,
            slider_tick_rate: 1.0,

            duration: 0,
            mins:0,
            secs:0,
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

