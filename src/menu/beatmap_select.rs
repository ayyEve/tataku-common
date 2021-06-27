use std::{path::Path, fs::read_dir};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use cgmath::Vector2;
use graphics::rectangle::Border;
use piston::{Key, MouseButton, RenderArgs};

use crate::databases::get_scores;
use crate::gameplay::{Beatmap, Score};
use crate::{WINDOW_SIZE, get_font, render::*};
use crate::game::{Game, GameMode, KeyModifiers, Settings};
use crate::menu::{Menu,ScoreMenu, ScrollableArea, ScrollableItem};
use crate::{DOWNLOADS_DIR, SONGS_DIR};

const INFO_BAR_HEIGHT:f64 = 60.0;
const BEATMAP_ITEM_SIZE: Vector2<f64> = Vector2::new(550.0, 50.0);
const BEATMAP_PAD_RIGHT: f64 = 5.0;
const LEADERBOARD_POS: Vector2<f64> = Vector2::new(10.0, 100.0);
const LEADERBOARD_ITEM_SIZE: Vector2<f64> = Vector2::new(200.0, 50.0);

pub struct BeatmapSelectMenu {
    /// hash of the selected map
    selected: Option<String>,
    beatmap_list: HashMap<String, Arc<Mutex<Beatmap>>>,

    // leaderboard: Arc<Mutex<Vec<Score>>>,
    
    current_scores: HashMap<String, Arc<Mutex<Score>>>,
    beatmap_scroll: ScrollableArea,
    leaderboard_scroll: ScrollableArea,

    background_texture: Option<Image>,

    pending_refresh: bool,
}
impl BeatmapSelectMenu {
    pub fn new() -> BeatmapSelectMenu {
        // get all folder names in the dir
        let folders = read_dir(SONGS_DIR).expect(&format!("\"{}\" dir does not exist", SONGS_DIR));
        let mut beatmap_list:HashMap<String, Arc<Mutex<Beatmap>>> = HashMap::new();
        let mut beatmap_scroll = ScrollableArea::new(Vector2::new(WINDOW_SIZE.x as f64 - (BEATMAP_ITEM_SIZE.x+BEATMAP_PAD_RIGHT), INFO_BAR_HEIGHT), Vector2::new(BEATMAP_ITEM_SIZE.x, WINDOW_SIZE.y as f64 - INFO_BAR_HEIGHT), true);
        
        for f in folders {
            let f = f.unwrap().path();
            let f = f.to_str().unwrap();
            if !Path::new(f).is_dir() {continue;}
            let dir_files = read_dir(f).unwrap();

            for file in dir_files {
                let file = file.unwrap().path();
                let file = file.to_str().unwrap();
                if file.ends_with(".osu") {
                    
                    let beatmap = Beatmap::load(file.to_owned());
                    let hash = beatmap.lock().unwrap().hash.clone();
                    beatmap_list.insert(hash, beatmap.clone());

                    let b = BeatmapItem::new(beatmap.clone());
                    beatmap_scroll.add_item(Box::new(b));

                    let mut b2 = Box::new(Vec::new());

                    b2.as_mut().push(1);
                }
            }
        }

        BeatmapSelectMenu {
            beatmap_list,
            selected: None,
            pending_refresh: false,
            current_scores: HashMap::new(),
            background_texture: None,
            // leaderboard: Arc::new(Mutex::new(Vec::new())),

            beatmap_scroll,
            leaderboard_scroll: ScrollableArea::new(LEADERBOARD_POS, Vector2::new(BEATMAP_ITEM_SIZE.x, WINDOW_SIZE.y as f64 - LEADERBOARD_POS.y), true),
        }
    }

    pub fn get_selected(&self) -> Option<Arc<Mutex<Beatmap>>> {
        if let Some(b) = self.beatmap_list.get(self.selected.as_ref().unwrap_or(&String::new())) {
            Some(b.clone())
        } else {None}
    }

    pub fn refresh_maps(&mut self) {
        self.pending_refresh = false;
        let folders = read_dir(SONGS_DIR).unwrap();

        for f in folders {
            let f = f.unwrap().path();
            let f = f.to_str().unwrap();
            if !Path::new(f).is_dir() {continue;}
            let dir_files = read_dir(f).unwrap();

            for file in dir_files {
                let file = file.unwrap().path();
                let file = file.to_str().unwrap();
                if file.ends_with(".osu") {
                    
                    let beatmap = Beatmap::load(file.to_owned());
                    let hash = beatmap.lock().unwrap().hash.clone();

                    // skip if its already added
                    if self.beatmap_list.contains_key(&hash) {continue}

                    self.beatmap_list.insert(hash, beatmap.clone());

                    let b = BeatmapItem::new(beatmap.clone());
                    self.beatmap_scroll.add_item(Box::new(b));

                    let mut b2 = Box::new(Vec::new());

                    b2.as_mut().push(1);
                }
            }
        }

    }

    pub fn load_scores(&mut self) {

        if let Some(map_hash) = &self.selected {
            // load scores
            let scores = get_scores(map_hash.to_owned());
            let scores = scores.lock().unwrap();
            self.leaderboard_scroll.clear();
            self.current_scores.clear();

            for s in scores.iter() {
                self.current_scores.insert(s.username.clone(), Arc::new(Mutex::new(s.clone())));
                self.leaderboard_scroll.add_item(Box::new(LeaderboardItem::new(s.to_owned())));
            }
        }
    }
}
impl Menu for BeatmapSelectMenu {
    fn update(&mut self, game:Arc<Mutex<&mut Game>>) {
        if game.lock().unwrap().beatmap_pending_refresh {
            game.lock().unwrap().beatmap_pending_refresh = false;
            self.pending_refresh = true;
            crate::game::extract_all();
        }

        if self.pending_refresh {
            let list = std::fs::read_dir(DOWNLOADS_DIR).unwrap();
            if list.count() <= 0 {
                self.refresh_maps();
            }
        }
    }

    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut items: Vec<Box<dyn Renderable>> = Vec::new();
        // let mut counter: usize = 0;
        let depth: f64 = 5.0;
        let font = crate::FONTS.get("main").unwrap().to_owned();

        // draw a bar on the top for the info
        let bar_rect = Rectangle::new(
            Color::WHITE,
            depth - 1.0,
            Vector2::new(0.0, 0.0),
            Vector2::new(args.window_size[0], INFO_BAR_HEIGHT),
            Some(Border {
                color: Color::BLACK.into(),
                radius: 1.2
            })
        );
        items.push(Box::new(bar_rect));

        // draw selected map info
        if let Some(b) = self.get_selected() {
            let b = b.lock().unwrap();
            let meta = b.metadata.clone();

            // draw map name top-most left-most
            let map_name = Text::new(
                Color::BLACK,
                -10.0,
                Vector2::new(0.0, 30.0),
                25,
                meta.version_string(),
                font.clone()
            );
            items.push(Box::new(map_name));

            // diff string, under map string
            let diff_string = Text::new(
                Color::BLACK,
                -10.0,
                Vector2::new(0.0, 55.0),
                15,
                meta.diff_string(),
                font.clone()
            );
            items.push(Box::new(diff_string));
        }
        

        // beatmap scroll
        items.extend(self.beatmap_scroll.draw(args));

        // leaderboard scroll
        items.extend(self.leaderboard_scroll.draw(args));

        // draw background image
        if let Some(img) = self.background_texture.as_ref() {
            items.push(Box::new(img.clone()));
        }

        items
    }

    fn on_volume_change(&mut self) {
        if let Some(item) = self.get_selected() {
            item.lock().unwrap().song.set_volume(Settings::get().get_music_vol());
        }
    }
    fn on_change(&mut self) {
        self.load_scores();
    }

    fn on_click(&mut self, pos:Vector2<f64>, button:MouseButton, game:Arc<Mutex<&mut Game>>) {

        if let Some(map_hash) = self.beatmap_scroll.on_click(pos, button, game.clone()) {

            // play selected map
            if let Some(map) = self.get_selected() {
                let mut map2 = map.lock().unwrap();
                if map2.hash == map_hash {
                    map2.song.stop();
                    map2.reset();
                    map2.start(); // TODO: figure out how to do this when checking mode change
                    game.lock().unwrap().start_map(map.clone());
                    return;
                }
            }

            // change selected map
            let mut changed=false;
            if let Some(map) = self.beatmap_list.get(&map_hash) {
                if let Some(selected) = self.get_selected() {
                    selected.lock().unwrap().song.stop();
                }
                changed = true;

                self.selected = Some(map.lock().unwrap().hash.clone());

                {
                    // play song 
                    let mut map = map.lock().unwrap();
                    map.song.set_volume(Settings::get().get_music_vol());
                    map.song.play();
                    // // load texture
                    // let meta = map.metadata.clone();
                    // if !meta.image_filename.is_empty() {
                    //     let settings = opengl_graphics::TextureSettings::new();
                    //     match opengl_graphics::Texture::from_path(meta.image_filename, &settings) {
                    //         Ok(tex) => {
                    //             self.background_texture = Some(Image::new(Vector2::new(0.0,0.0), f64::MIN, tex));
                    //         },
                    //         Err(e) => println!("error reading tex: {}", e),
                    //     }
                    // }
                }
            }
            
            if changed {self.load_scores()}
            return;
        }

        if let Some(score_tag) = self.leaderboard_scroll.on_click(pos, button, game.clone()) {
            // score display
            let mut game = game.lock().unwrap();
            if let Some(score) = self.current_scores.get(&score_tag) {
                let score = score.lock().unwrap().clone();
                let menu = ScoreMenu::new(score);
                game.queue_mode_change(GameMode::InMenu(Arc::new(Mutex::new(Box::new(menu)))));
            }
        }
        
    }
    fn on_mouse_move(&mut self, pos:Vector2<f64>, game:Arc<Mutex<&mut Game>>) {
        self.beatmap_scroll.on_mouse_move(pos, game.clone());
        self.leaderboard_scroll.on_mouse_move(pos, game.clone());
    }
    fn on_scroll(&mut self, delta:f64, _game:Arc<Mutex<&mut Game>>) {
        self.beatmap_scroll.on_scroll(delta);
        self.leaderboard_scroll.on_scroll(delta);
    }

    fn on_key_press(&mut self, key:piston::Key, game:Arc<Mutex<&mut Game>>, _mods:KeyModifiers) {
        let mut game = game.lock().unwrap();
        if key == Key::Escape {
            let menu = game.menus.get("main").unwrap().clone();
            game.queue_mode_change(GameMode::InMenu(menu));
        }
        if key == Key::F5 {
            self.refresh_maps();
        }
    }

    //TODO: implement search (oh god)
    fn on_text(&mut self, _text:String) {
        
    }
}

/// more like BeatmapsetItem
struct BeatmapItem {
    //TODO: make this have BeatmapMeta, and a vec of beatmaps. this button should represent the set
    beatmap: Arc<Mutex<Beatmap>>,
    pos: Vector2<f64>,

    hover: bool,
    selected: bool,

    tag:String,
}
impl BeatmapItem {
    fn new(beatmap: Arc<Mutex<Beatmap>>) -> BeatmapItem {
        let tag = beatmap.lock().unwrap().hash.clone();
        BeatmapItem {
            beatmap, 
            pos: Vector2::new(0.0,0.0),
            hover: false,
            selected: false,
            tag
        }
    }
}
impl ScrollableItem for BeatmapItem {
    fn size(&self) -> Vector2<f64> {BEATMAP_ITEM_SIZE}
    fn get_tag(&self) -> String {self.tag.clone()}
    fn set_tag(&mut self, tag:String) {self.tag = tag}
    fn get_pos(&self) -> Vector2<f64> {self.pos}
    fn set_pos(&mut self, pos:Vector2<f64>) {self.pos = pos}

    fn draw(&mut self, _args:RenderArgs, pos_offset:Vector2<f64>) -> Vec<Box<dyn Renderable>> {
        let mut items: Vec<Box<dyn Renderable>> = Vec::new();
        let meta = self.beatmap.lock().unwrap().metadata.clone();
        let font = crate::get_font("main");

        let depth = 5.0;

        // draw rectangle
        items.push(Box::new(Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            depth,
            self.pos+pos_offset,
            BEATMAP_ITEM_SIZE,
            if self.hover {
                Some(Border {color: Color::RED.into(),radius: 1.0})
            } else if self.selected {
                Some(Border {color: Color::BLUE.into(),radius: 1.0})
            } else {
                None
            }
        )));

        // line 1
        items.push(Box::new(Text::new(
            Color::WHITE,
            depth - 1.0,
            self.pos+pos_offset + Vector2::new(5.0, 20.0),
            15,
            format!("{} // {} - {}",meta.creator, meta.artist, meta.title),
            font.clone()
        )));

        // lline 2
        items.push(Box::new(Text::new(
            Color::WHITE,
            depth - 1.0,
            self.pos+pos_offset + Vector2::new(5.0, 40.0),
            12,
            format!("{}", meta.version),
            font.clone()
        )));

        items
    }

    fn on_click(&mut self, _pos:Vector2<f64>, _button:MouseButton) -> bool {
        self.selected = self.hover;
        self.hover
    }
    fn on_mouse_move(&mut self, pos:Vector2<f64>) {self.hover = self.hover(pos)}
}


struct LeaderboardItem {
    pos: Vector2<f64>,
    score: Score,

    tag: String,
    hover: bool,

    acc:f64,
}
impl LeaderboardItem {
    pub fn new(score:Score) -> LeaderboardItem {
        let tag = score.username.clone();
        let acc = score.acc() * 100.0;

        LeaderboardItem {
            pos: Vector2::new(0.0,0.0),
            score,
            tag,
            acc,
            hover: false
        }
    }
}
impl ScrollableItem for LeaderboardItem {
    fn size(&self) -> Vector2<f64> {LEADERBOARD_ITEM_SIZE}
    fn get_tag(&self) -> String {self.tag.clone()}
    fn set_tag(&mut self, tag:String) {self.tag = tag}
    fn get_pos(&self) -> Vector2<f64> {self.pos}
    fn set_pos(&mut self, pos:Vector2<f64>) {self.pos = pos}

    fn draw(&mut self, _args:RenderArgs, pos_offset:Vector2<f64>) -> Vec<Box<dyn Renderable>> {
        let mut items: Vec<Box<dyn Renderable>> = Vec::new();
        let font = get_font("main");
        
        let depth = 5.0;

        // bounding rect
        let area = Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            depth,
            self.pos+pos_offset,
            LEADERBOARD_ITEM_SIZE,
            if self.hover {
                Some(Border {color: Color::RED.into(), radius: 1.0})
            } else {
                None
            }
        );
        items.push(Box::new(area));

        // score text
        let text = Text::new(
            Color::WHITE,
            depth - 1.0,
            self.pos+pos_offset + Vector2::new(5.0, 20.0),
            15,
            format!("{}: {}", self.score.username, crate::format(self.score.score)),
            font.clone()
        );
        items.push(Box::new(text));

        // combo text
        let text = Text::new(
            Color::WHITE,
            depth - 1.0,
            self.pos+pos_offset + Vector2::new(5.0, 40.0),
            12,
            format!("{}x, {:.2}%", crate::format(self.score.max_combo), self.acc),
            font.clone()
        );
        items.push(Box::new(text));

        items
    }

    fn on_click(&mut self, _pos:Vector2<f64>, _button:MouseButton) -> bool {self.hover}
    fn on_mouse_move(&mut self, pos:Vector2<f64>) {self.hover = self.hover(pos);}
}
