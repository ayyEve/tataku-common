use std::{path::Path, sync::{Arc, Weak}, time::Instant};

use piston::RenderArgs;
use parking_lot::Mutex;

use taiko_rs_common::types::{KeyPress, Replay, Score, ScoreHit};
use super::{*, diff_calc::DifficultyCalculator, beatmap_structs::*};
use crate::{HIT_AREA_RADIUS, HIT_POSITION, PLAYFIELD_RADIUS, WINDOW_SIZE, game::{Audio, AudioHandle, Settings}};
use crate::{NOTE_RADIUS, enums::Playmode, game::{get_font, Vector2}, render::{Renderable, Rectangle, Text, Circle, Color, Border}};

const LEAD_IN_TIME:f32 = 1000.0; // how much time should pass at beatmap start before audio begins playing (and the map "starts")
pub const BAR_COLOR:Color = Color::new(0.0, 0.0, 0.0, 1.0); // timing bar color
const BAR_WIDTH:f64 = 4.0; // how wide is a timing bar
const BAR_SPACING:f64 = 4.0; // how many beats between timing bars
const SV_FACTOR:f64 = 700.0; // bc sv is bonked, divide it by this amount
const DURATION_HEIGHT:f64 = 35.0; // how tall is the duration bar
const OFFSET_DRAW_TIME:i64 = 2_000; // how long should the offset be drawn for?

const HIT_TIMING_BAR_SIZE:Vector2 = Vector2::new(WINDOW_SIZE.x / 3.0, 30.0);
const HIT_TIMING_BAR_POS:Vector2 = Vector2::new(WINDOW_SIZE.x / 2.0 - HIT_TIMING_BAR_SIZE.x / 2.0, WINDOW_SIZE.y - (DURATION_HEIGHT + 3.0 + HIT_TIMING_BAR_SIZE.y + 5.0));
const HIT_TIMING_DURATION:f64 = 1_000.0; // how long should a hit timing line last
const HIT_TIMING_FADE:f64 = 300.0; // how long to fade out for
const HIT_TIMING_BAR_COLOR:Color = Color::new(0.0, 0.0, 0.0, 1.0); // hit timing bar color

#[derive(Clone)]
pub struct Beatmap {
    pub score: Option<Score>,
    pub replay: Option<Replay>,

    pub hash: String,
    pub started: bool,
    pub completed: bool,
    
    // lists
    pub notes: Arc<Mutex<Vec<Box<dyn HitObject>>>>,
    pub timing_points: Vec<TimingPoint>,
    timing_bars: Vec<TimingBar>,
    // list indices
    note_index: usize,
    timing_point_index: usize,

    song_start: Instant,
    song: Weak<AudioHandle>,
    lead_in_time: f32,
    end_time: f64,

    // offset things
    offset: i64,
    offset_changed_time: i64,

    // meta info
    pub metadata: BeatmapMeta,

    // hit timing bar stuff
    /// map time, diff (note - hit) //TODO: figure out how to draw this efficiently
    hit_timings: Vec<(i64, i64)>,
    hitwindow_300: f64,
    hitwindow_100: f64,
    hitwindow_miss: f64
}
impl Beatmap {
    pub fn load(dir:String) -> Arc<Mutex<Beatmap>> {
        let lines = crate::read_lines(dir.clone()).expect("Beatmap file not found");
        let mut body = String::new();
        let mut current_area = BeatmapSection::Version;
        let mut beatmap = Beatmap {
            hash: String::new(),
            notes: Arc::new(Mutex::new(Vec::new())),
            timing_points: Vec::new(),
            timing_bars: Vec::new(),
            song_start: Instant::now(),
            score: None,
            replay: None,
            metadata: BeatmapMeta::new(),
            song: Weak::new(), // temp until we get the audio file path
            note_index: 0,
            timing_point_index: 0,
            started: false,
            completed: false,
            end_time: 0.0,
            lead_in_time: LEAD_IN_TIME,

            offset: 0,
            offset_changed_time: 0,

            hit_timings: Vec::new(),
            hitwindow_100: 0.0,
            hitwindow_300: 0.0,
            hitwindow_miss: 0.0,
        };

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
                        let tp = TimingPoint::from_str(&line, tp_parent.clone());

                        if !tp.is_inherited() {
                            tp_parent = Some(Arc::new(tp.clone()));
                        }

                        beatmap.timing_points.push(tp);
                    }
                    BeatmapSection::HitObjects => {
                        let mut split = line.split(",");
                        if split.clone().count() < 2 {continue} // skip empty lines

                        let _x = split.next();
                        let _y = split.next();
                        let time = split.next().unwrap().parse::<u64>().unwrap();
                        let read_type = split.next().unwrap().parse::<u64>().unwrap_or(0); // note, slider, spinner
                        let hitsound = split.next().unwrap().parse::<u32>().unwrap_or(0); // 0 = normal, 2 = whistle, 4 = finish, 8 = clap

                        let hit_type = if (hitsound & (2 | 8)) > 0 {super::HitType::Kat} else {super::HitType::Don};
                        let finisher = (hitsound & 4) > 0;
                        
                        // set later, bc for some reason its inconsistant here
                        let sv = 1.0; //beatmap.slider_velocity_at(time) / SV_FACTOR;

                        if (read_type & 2) > 0 { // slider
                            let _curve = split.next(); // dont care
                            let slides = split.next().unwrap().parse::<u64>().unwrap();
                            let length = split.next().unwrap().parse::<f64>().unwrap();

                            let l = (length * 1.4) * slides as f64;
                            let v2 = 100.0 * (beatmap.metadata.slider_multiplier as f64 * 1.4);
                            let bl = beatmap.beat_length_at(time as f64, true);
                            let end_time = time + (l / v2 * bl) as u64;
                            
                            // convert vars
                            let v = beatmap.slider_velocity_at(time);
                            let bl = beatmap.beat_length_at(time as f64, beatmap.metadata.beatmap_version < 8.0);
                            let skip_period = (bl / beatmap.metadata.slider_tick_rate as f64).min((end_time - time) as f64 / slides as f64);

                            if skip_period > 0.0 && beatmap.metadata.mode != Playmode::Taiko && l / v * 1000.0 < 2.0 * bl {
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

                                //TODO: could this be turned into a for i in (x..y).step(n) ?
                                loop {
                                    let sound_type = sound_types[i];

                                    let note = Note::new(
                                        j as u64,
                                        sound_type.0,
                                        sound_type.1,
                                        sv
                                    );
                                    beatmap.notes.lock().push(Box::new(note));

                                    if !unified_sound_addition {i = (i + 1) % sound_types.len()}

                                    j += skip_period;
                                    if !(j < end_time as f64 + skip_period / 8.0) {break}
                                }
                            } else {
                                let slider = Slider::new(time, end_time, finisher, sv);
                                beatmap.notes.lock().push(Box::new(slider));
                            }

                        } else if (read_type & 8) > 0 { // spinner
                            // x,y,time,type,hitSound,...
                            // endTime,hitSample
                            let end_time = split.next().unwrap().parse::<u64>().unwrap();
                            let length = end_time as f64 - time as f64;

                            let diff_map = map_difficulty_range(beatmap.metadata.od as f64, 3.0, 5.0, 7.5);
                            let hits_required:u16 = ((length / 1000.0 * diff_map) * 1.65).max(1.0) as u16; // ((this.Length / 1000.0 * this.MapDifficultyRange(od, 3.0, 5.0, 7.5)) * 1.65).max(1.0)
                            // just make a slider for now
                            let spinner = Spinner::new(time, end_time, sv, hits_required);
                            beatmap.notes.lock().push(Box::new(spinner));
                        } else { // note
                            let note = Note::new(time, hit_type, finisher, sv);
                            beatmap.notes.lock().push(Box::new(note));
                        }
                    }

                    // dont need to do anything with these for the scope of this game
                    BeatmapSection::Editor => {},
                    BeatmapSection::Colors => {},
                }
            }
        }

        beatmap.notes.lock().sort_by(|a, b| a.time().cmp(&b.time()));
        let start_time = beatmap.notes.lock().first().unwrap().time() as f64;
        let end_time = beatmap.notes.lock().last().unwrap().end_time(0.0) as f64;

        beatmap.hash = format!("{:x}", md5::compute(body).to_owned());
        beatmap.metadata.set_dur((end_time - start_time) as u64);
        beatmap.end_time = end_time;

        // this might be an issue later on *maybe*
        beatmap.calc_sr();
        Arc::new(Mutex::new(beatmap))
    }

    pub fn calc_sr(&mut self) {self.metadata.sr = DifficultyCalculator::new(Arc::new(Mutex::new(self.to_owned()))).compute_difficulty()}
    pub fn time(&self) -> i64 {self.song.upgrade().unwrap().current_time() as i64 - (self.lead_in_time as i64 + self.offset)}
    pub fn next_note(&mut self) {self.note_index += 1}
    pub fn increment_offset(&mut self, delta:i64) {
        self.offset += delta;
        self.offset_changed_time = self.time();
    }

    pub fn hit(&mut self, key:KeyPress) {
        let time = self.time() as f64;
        if let Some(replay) = self.replay.as_mut() {replay.presses.push((time as i64, key))}

        let hit_type:HitType = key.into();
        let mut sound = match hit_type {HitType::Don => "don", HitType::Kat => "kat"};

        let notes = self.notes.clone();
        let mut notes = notes.lock();

        let hit_volume = Settings::get().get_effect_vol() * (self.timing_points[self.timing_point_index].volume as f32 / 100.0);

        // if theres no more notes to hit, return
        if self.note_index >= notes.len() {
            let a = Audio::play_preloaded(sound);
            a.upgrade().unwrap().set_volume(hit_volume);
            return;
        }

        // check for finisher 2nd hit. 
        if self.note_index > 0 {
            let last_note = notes.get_mut(self.note_index-1).unwrap();

            match last_note.check_finisher(hit_type, time) {
                ScoreHit::Miss => {return},
                ScoreHit::X100 => {
                    self.score.as_mut().unwrap().add_pts(100, true);
                    return;
                },
                ScoreHit::X300 => {
                    self.score.as_mut().unwrap().add_pts(300, true);
                    return;
                },
                ScoreHit::Other(points, _) => {
                    self.score.as_mut().unwrap().add_pts(points as u64, false);
                    return;
                },
                ScoreHit::None => {},
            }
        }

        let note = notes.get_mut(self.note_index).unwrap();
        let note_time = note.time() as f64;

        match note.get_points(hit_type, time, (self.hitwindow_miss, self.hitwindow_100, self.hitwindow_300)) {
            ScoreHit::None => {
                // play sound
                // Audio::play_preloaded(sound);
            },
            ScoreHit::Miss => {
                self.score.as_mut().unwrap().hit_miss(time as u64, note_time as u64);
                self.hit_timings.push((time as i64, (time - note_time) as i64));
                self.next_note();
                // Audio::play_preloaded(sound);

                //TODO: play miss sound
                //TODO: indicate this was a miss
            },
            ScoreHit::X100 => {
                self.score.as_mut().unwrap().hit100(time as u64, note_time as u64);
                self.hit_timings.push((time as i64, (time - note_time) as i64));

                // only play finisher sounds if the note is both a finisher and was hit
                // could maybe also just change this to HitObject.get_sound() -> &str
                if note.finisher_sound() {sound = match hit_type {HitType::Don => "bigdon", HitType::Kat => "bigkat"};}
                // Audio::play_preloaded(sound);
                //TODO: indicate this was a bad hit

                self.next_note();
            },
            ScoreHit::X300 => {
                self.score.as_mut().unwrap().hit300(time as u64, note_time as u64);
                self.hit_timings.push((time as i64, (time - note_time) as i64));
                
                if note.finisher_sound() {sound = match hit_type {HitType::Don => "bigdon",HitType::Kat => "bigkat"};}
                // Audio::play_preloaded(sound);

                self.next_note();
            },
            ScoreHit::Other(score, consume) => { // used by sliders and spinners
                self.score.as_mut().unwrap().score += score as u64;
                if consume {self.next_note()}
                // Audio::play_preloaded(sound);
            }
        }

        let a = Audio::play_preloaded(sound);
        a.upgrade().unwrap().set_volume(hit_volume);
    }

    pub fn update(&mut self) {
        // check lead-in time
        if self.lead_in_time > 0.0 {
            let elapsed = self.song_start.elapsed().as_micros() as f32 / 1000.0;
            self.song_start = Instant::now();
            self.lead_in_time -= elapsed;

            if self.lead_in_time <= 0.0 {
                let song = self.song.upgrade().unwrap();
                song.set_position(-self.lead_in_time);
                song.set_volume(Settings::get().get_music_vol());
                song.play();
                self.lead_in_time = 0.0;
            }
        }

        // get the current time
        let time = self.time();

        // update notes
        for note in self.notes.lock().iter_mut() {note.update(time)}

        // update hit timings bar
        self.hit_timings.retain(|(hit_time, _)| {time - hit_time < HIT_TIMING_DURATION as i64});

        // if theres no more notes to hit, show score screen
        if self.note_index >= self.notes.lock().len() {
            self.completed = true;
            return;
        }

        // check if we missed the current note
        if (self.notes.lock()[self.note_index].end_time(self.hitwindow_miss) as i64) < time {
            if self.notes.lock()[self.note_index].causes_miss() {
                // need to set these manually instead of score.hit_miss,
                // since we dont want to add anything to the hit error list
                let s = self.score.as_mut().unwrap();
                s.xmiss += 1;
                s.combo = 0;
            }
            self.next_note();
        }
        
        // TODO: might move tbs to a (time, speed) tuple
        for tb in self.timing_bars.iter_mut() {tb.update(time as f64)}

        // check timing point
        if self.timing_point_index + 1 < self.timing_points.len() && self.timing_points[self.timing_point_index + 1].time <= time as f64 {
            self.timing_point_index += 1;
        }
    }
    pub fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut renderables: Vec<Box<dyn Renderable>> = Vec::new();
        // load this here, it a bit more performant
        let font = get_font("main");
        let score = self.score.as_ref().unwrap();
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
        renderables.push(Box::new(Circle::new(
            Color::BLACK,
            f64::MAX,
            HIT_POSITION,
            HIT_AREA_RADIUS + 2.0
        )));

        // score text
        renderables.push(Box::new(Text::new(
            Color::BLACK,
            0.0,
            Vector2::new(args.window_size[0] - 200.0, 40.0),
            30,
            crate::format(score.score),
            font.clone()
        )));

        // acc text
        renderables.push(Box::new(Text::new(
            Color::BLACK,
            0.0,
            Vector2::new(args.window_size[0] - 200.0, 70.0),
            30,
            format!("{:.2}%", score.acc()*100.0),
            font.clone()
        )));

        // combo text
        let mut combo_text = Text::new(
            Color::WHITE,
            0.0,
            HIT_POSITION - Vector2::new(100.0, 0.0),
            30,
            crate::format(score.combo),
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
            offset_text.center_text(Rectangle::bounds_only(Vector2::zero(), Vector2::new(WINDOW_SIZE.x , HIT_POSITION.y)));
            renderables.push(Box::new(offset_text));
        }

        // duration bar
        // duration remaining
        renderables.push(Box::new(Rectangle::new(
            Color::TRANSPARENT_WHITE,
            1.0,
            Vector2::new(0.0, args.window_size[1] - (DURATION_HEIGHT + 3.0)),
            Vector2::new(args.window_size[0], DURATION_HEIGHT),
            Some(Border::new(Color::BLACK, 1.8))
        )));
        // fill
        renderables.push(Box::new(Rectangle::new(
            [0.4,0.4,0.4,1.0].into(),
            2.0,
            Vector2::new(0.0, args.window_size[1] - (DURATION_HEIGHT + 3.0)),
            Vector2::new(args.window_size[0] * (self.time() as f64/self.end_time), DURATION_HEIGHT),
            None
        )));


        // draw hit timings bar
        // draw hit timing colors below the bar
        let width_300 = self.hitwindow_300 / self.hitwindow_miss * HIT_TIMING_BAR_SIZE.x;
        let width_100 = self.hitwindow_100 / self.hitwindow_miss * HIT_TIMING_BAR_SIZE.x;
        let width_miss = self.hitwindow_miss / self.hitwindow_miss * HIT_TIMING_BAR_SIZE.x;

        renderables.push(Box::new(Rectangle::new(
            [0.1960, 0.7372, 0.9058, 1.0].into(),
            17.0,
            Vector2::new(WINDOW_SIZE.x/ 2.0 - width_300/2.0, HIT_TIMING_BAR_POS.y),
            Vector2::new(width_300, HIT_TIMING_BAR_SIZE.y),
            None // for now
        )));
        renderables.push(Box::new(Rectangle::new(
            [0.3411, 0.8901, 0.07450, 1.0].into(),
            18.0,
            Vector2::new(WINDOW_SIZE.x / 2.0 - width_100/2.0, HIT_TIMING_BAR_POS.y),
            Vector2::new(width_100, HIT_TIMING_BAR_SIZE.y),
            None // for now
        )));
        renderables.push(Box::new(Rectangle::new(
            [0.8549, 0.6823, 0.2745, 1.0].into(),
            19.0,
            Vector2::new(WINDOW_SIZE.x  / 2.0 - width_miss/2.0, HIT_TIMING_BAR_POS.y),
            Vector2::new(width_miss, HIT_TIMING_BAR_SIZE.y),
            None // for now
        )));

        // draw hit timings
        let time = time as f64;
        for (hit_time, diff) in self.hit_timings.as_slice() {
            let hit_time = hit_time.clone() as f64;
            let mut diff = diff.clone() as f64;
            if diff < 0.0 {
                diff = diff.max(-self.hitwindow_miss);
            } else {
                diff = diff.min(self.hitwindow_miss);
            }

            let pos = diff / self.hitwindow_miss * (HIT_TIMING_BAR_SIZE.x / 2.0);

            // draw diff line
            let diff = time - hit_time;
            let alpha = if diff > HIT_TIMING_DURATION - HIT_TIMING_FADE {
                1.0 - (diff - (HIT_TIMING_DURATION - HIT_TIMING_FADE)) / HIT_TIMING_FADE
            } else {1.0};

            let mut c = HIT_TIMING_BAR_COLOR;
            c.a = alpha as f32;
            renderables.push(Box::new(Rectangle::new(
                c,
                10.0,
                Vector2::new(WINDOW_SIZE.x  / 2.0 + pos, HIT_TIMING_BAR_POS.y),
                Vector2::new(2.0, HIT_TIMING_BAR_SIZE.y),
                None // for now
            )));
        }

        // draw notes
        for note in self.notes.lock().iter_mut() {renderables.extend(note.draw(args));}
        // draw timing lines
        for tb in self.timing_bars.iter_mut() {renderables.extend(tb.draw(args))}

        renderables
    }

    // can be from either paused or new
    pub fn start(&mut self) {
        if !self.started {
            self.song.upgrade().unwrap().set_position(0.0);
            self.lead_in_time = LEAD_IN_TIME;
            self.song_start = Instant::now();
            // volume is set when the song is actually started (when lead_in_time is <= 0)
            self.started = true;
            return;
        } else if self.lead_in_time <= 0.0 {
            self.song.upgrade().unwrap().play();
        }
    }
    pub fn pause(&mut self) {
        self.song.upgrade().unwrap().pause();
        // is there anything else we need to do?

        // might mess with lead-in but meh
    }
    pub fn reset(&mut self) {
        let settings = Settings::get().clone();
        
        // reset song
        match self.song.upgrade().clone() {
            Some(song) => {
                song.set_position(0.0);
                song.pause();
            },
            None => {
                self.song = Audio::play_song(self.metadata.audio_filename.clone(), true);
                let s = self.song.upgrade().unwrap();
                s.pause();
            },
        }

        let c = self.clone();
        for note in self.notes.lock().as_mut_slice() {
            note.reset();

            // set note svs
            if settings.static_sv {
                note.set_sv(settings.sv_multiplier as f64);
            } else {
                let sv = c.slider_velocity_at(note.time()) / SV_FACTOR;
                note.set_sv(sv);
            }
        }
        
        self.note_index = 0;
        self.timing_point_index = 0;
        self.completed = false;
        self.started = false;
        self.lead_in_time = LEAD_IN_TIME;
        self.offset_changed_time = 0;
        self.song_start = Instant::now();

        // setup hitwindows
        self.hitwindow_miss = map_difficulty_range(self.metadata.od as f64, 135.0, 95.0, 70.0);
        self.hitwindow_100 = map_difficulty_range(self.metadata.od as f64, 120.0, 80.0, 50.0);
        self.hitwindow_300 = map_difficulty_range(self.metadata.od as f64, 50.0, 35.0, 20.0);

        // setup timing bars
        //TODO: it would be cool if we didnt actually need timing bar objects, and could just draw them
        if self.timing_bars.len() == 0 {
            // load timing bars
            let parent_tps = self.timing_points.iter().filter(|t|!t.is_inherited()).collect::<Vec<&TimingPoint>>();
            let mut sv = settings.sv_multiplier as f64;
            let mut time = parent_tps[0].time;
            let mut tp_index = 0;
            let step = self.beat_length_at(time, false);
            time %= step; // get the earliest bar line possible

            loop {
                if !settings.static_sv {sv = self.slider_velocity_at(time as u64) / SV_FACTOR}

                // if theres a bpm change, adjust the current time to that of the bpm change
                let next_bar_time = self.beat_length_at(time, false) * BAR_SPACING; // bar spacing is actually the timing point measure

                // edge case for aspire maps
                if next_bar_time.is_nan() || next_bar_time == 0.0 {
                    break;
                }

                // add timing bar at current time
                self.timing_bars.push(TimingBar::new(time as u64, sv));

                if tp_index < parent_tps.len() && parent_tps[tp_index].time <= time + next_bar_time {
                    time = parent_tps[tp_index].time;
                    tp_index += 1;
                    continue;
                }

                // why isnt this accounting for bpm changes? because the bpm change doesnt allways happen inline with the bar idiot
                time += next_bar_time;
                if time >= self.end_time || time.is_nan() {break}
            }

            println!("created {} timing bars", self.timing_bars.len());
        }
    
        self.score = Some(Score::new(self.hash.clone(), settings.username.clone()));
        self.replay = Some(Replay::new());
    }
    pub fn cleanup(&mut self) {
        self.timing_bars.clear();
        self.hit_timings.clear();
        self.score = None;
        self.replay = None;
    }

    pub fn skip_intro(&mut self) {
        if self.note_index > 0 {return}

        let x_needed = WINDOW_SIZE.x;
        let mut time = self.time();

        let notes = self.notes.lock();

        loop {
            let mut found = false;
            for note in notes.iter() {if note.x_at(time) <= x_needed {found = true; break}}
            if found {break}
            time += 1;
        }

        let mut time = time as f32;
        if self.lead_in_time > 0.0 {
            if time > self.lead_in_time {
                time -= self.lead_in_time - 0.01;
                self.lead_in_time = 0.01;
            }
        }

        self.song.upgrade().unwrap().set_position(time);
    }


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
}


// timing bar struct
//TODO: might be able to reduce this to a (time, speed) and just calc pos on draw
#[derive(Copy, Clone, Debug)]
struct TimingBar {
    time: u64,
    speed: f64,
    pos: Vector2
}
impl TimingBar {
    pub fn new(time:u64, speed:f64) -> TimingBar {
        TimingBar {
            time, 
            speed,
            pos: Vector2::zero(),
        }
    }

    pub fn update(&mut self, time:f64) {
        self.pos = HIT_POSITION + Vector2::new(((self.time as f64 - time as f64) * self.speed) - BAR_WIDTH / 2.0, -PLAYFIELD_RADIUS);
    }

    fn draw(&mut self, _args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut renderables: Vec<Box<dyn Renderable>> = Vec::new();
        if self.pos.x + BAR_WIDTH < 0.0 || self.pos.x - BAR_WIDTH > WINDOW_SIZE.x as f64 {return renderables}

        const SIZE:Vector2 = Vector2::new(BAR_WIDTH, PLAYFIELD_RADIUS*2.0);
        const DEPTH:f64 = f64::MAX-5.0;

        renderables.push(Box::new(Rectangle::new(
            BAR_COLOR,
            DEPTH,
            self.pos,
            SIZE,
            None
        )));

        renderables
    }
}


// contains beatmap info unrelated to notes and timing points, etc
#[derive(Clone, Debug)]
pub struct BeatmapMeta {
    pub mode: Playmode,
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


impl Into<HitType> for KeyPress {
    fn into(self) -> HitType {
        match self {
            KeyPress::LeftKat|KeyPress::RightKat => HitType::Kat,
            KeyPress::LeftDon|KeyPress::RightDon => HitType::Don,
        }
    }
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