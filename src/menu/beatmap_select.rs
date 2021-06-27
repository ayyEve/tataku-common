use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::{path::Path, fs::read_dir};

use cgmath::Vector2;
use graphics::rectangle::Border;
use piston::{Key, MouseButton, RenderArgs};

use crate::databases::get_scores;
use crate::gameplay::{Beatmap, BeatmapMeta, Score};
use crate::game::{Game, GameMode, KeyModifiers, Settings};
use crate::menu::{Menu,ScoreMenu, ScrollableArea, ScrollableItem};
use crate::{WINDOW_SIZE, DOWNLOADS_DIR, SONGS_DIR, get_font, render::*};

const INFO_BAR_HEIGHT:f64 = 60.0;
const BEATMAPSET_ITEM_SIZE: Vector2<f64> = Vector2::new(550.0, 50.0);
const BEATMAPSET_PAD_RIGHT: f64 = 5.0;

const BEATMAP_ITEM_PADDING: f64 = 5.0;
const BEATMAP_ITEM_SIZE: Vector2<f64> = Vector2::new(450.0, 50.0);

const LEADERBOARD_POS: Vector2<f64> = Vector2::new(10.0, 100.0);
const LEADERBOARD_ITEM_SIZE: Vector2<f64> = Vector2::new(200.0, 50.0);


pub struct BeatmapSelectMenu {
    /// hash of the selected map
    selected: Option<String>,

    // leaderboard: Arc<Mutex<Vec<Score>>>,
    
    current_scores: HashMap<String, Arc<Mutex<Score>>>,
    beatmap_scroll: ScrollableArea,
    leaderboard_scroll: ScrollableArea,

    background_texture: Option<Image>,

    pending_refresh: bool,
}
impl BeatmapSelectMenu {
    pub fn new() -> BeatmapSelectMenu {

        let mut b = BeatmapSelectMenu {
            selected: None,
            pending_refresh: false,
            current_scores: HashMap::new(),
            background_texture: None,

            beatmap_scroll: ScrollableArea::new(Vector2::new(WINDOW_SIZE.x as f64 - (BEATMAPSET_ITEM_SIZE.x+BEATMAPSET_PAD_RIGHT), INFO_BAR_HEIGHT), Vector2::new(BEATMAPSET_ITEM_SIZE.x, WINDOW_SIZE.y as f64 - INFO_BAR_HEIGHT), true),
            leaderboard_scroll: ScrollableArea::new(LEADERBOARD_POS, Vector2::new(BEATMAPSET_ITEM_SIZE.x, WINDOW_SIZE.y as f64 - LEADERBOARD_POS.y), true),
        };
        b.refresh_maps();
        b
    }

    /// returns the selected item
    pub fn get_selected(&self) -> Option<(Arc<Mutex<Beatmap>>, bool)> {
        if self.selected.is_none() {return None}
        let s = self.beatmap_scroll.get_tagged(self.selected.as_ref().unwrap().clone()).first().unwrap().get_value();
        let s = s.downcast_ref::<(Arc<Mutex<Beatmap>>, bool)>();
        if let Some(b) = s {Some(b.clone())} else {None}
    }

    pub fn refresh_maps(&mut self) {
        self.pending_refresh = false;
        self.beatmap_scroll.clear();
        let folders = read_dir(SONGS_DIR).unwrap();

        for f in folders {
            let f = f.unwrap().path();
            let f = f.to_str().unwrap();
            if !Path::new(f).is_dir() {continue;}
            let dir_files = read_dir(f).unwrap();
            let mut list = Vec::new();

            for file in dir_files {
                let file = file.unwrap().path();
                let file = file.to_str().unwrap();

                if file.ends_with(".osu") {
                    list.push(Beatmap::load(file.to_owned()));
                }
            }

            if list.len() > 0 {
                self.beatmap_scroll.add_item(Box::new(BeatmapsetItem::new(list)));
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
        if let Some((b, _play)) = self.get_selected() {
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

    fn on_volume_change(&mut self) {self.beatmap_scroll.on_volume_change();}
    fn on_change(&mut self) {
        self.beatmap_scroll.refresh_layout();
        self.load_scores();
    }

    fn on_click(&mut self, pos:Vector2<f64>, button:MouseButton, game:Arc<Mutex<&mut Game>>) {

        if let Some(clicked_tag) = self.beatmap_scroll.on_click(pos, button, game.clone()) {
            let clicked = self.beatmap_scroll.get_tagged(clicked_tag.clone()).first().unwrap().get_value();
            let (clicked, play) = clicked.downcast_ref::<(Arc<Mutex<Beatmap>>, bool)>().unwrap();

            if *play {
                for i in self.beatmap_scroll.items.as_mut_slice() {
                    i.set_tag(String::new());
                }
                let mut map = clicked.lock().unwrap();
                map.song.stop();
                map.reset();
                map.start(); // TODO: figure out how to do this when checking mode change
                game.lock().unwrap().start_map(clicked.clone());
                return;
            }

            // get curremnt selected map
            if let Some((b, _play)) = self.get_selected() {
                b.lock().unwrap().song.stop();
            }

            self.selected = Some(clicked_tag.clone());
            self.beatmap_scroll.refresh_layout();

            self.load_scores();
            return;
        } else {
            self.selected = None;
            self.beatmap_scroll.refresh_layout();
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


struct BeatmapsetItem {
    //TODO: make this have BeatmapMeta, and a vec of beatmaps. this button should represent the set
    beatmaps: Vec<Arc<Mutex<Beatmap>>>,
    pos: Vector2<f64>,

    hover: bool,
    selected: bool,
    pending_play: bool,

    tag: String,
    meta: BeatmapMeta,
    selected_item: usize, // index of selected item
    mouse_pos:Vector2<f64>,

    // use this for audio
    first: Arc<Mutex<Beatmap>>
}
impl BeatmapsetItem {
    fn new(beatmaps: Vec<Arc<Mutex<Beatmap>>>) -> BeatmapsetItem {
        let _first = beatmaps.first().unwrap();
        let first = _first.lock().unwrap();
        let tag = first.metadata.version_string();

        BeatmapsetItem {
            beatmaps:beatmaps.clone(), 
            pos: Vector2::new(0.0,0.0),
            hover: false,
            selected: false,
            pending_play: false,
            tag,
            meta: first.metadata.clone(),

            selected_item: 0,
            first: _first.clone(),
            mouse_pos: Vector2::new(0.0,0.0)
        }
    }
}
impl ScrollableItem for BeatmapsetItem {
    fn size(&self) -> Vector2<f64> {
        if !self.selected {
            BEATMAPSET_ITEM_SIZE
        } else {
            Vector2::new(BEATMAPSET_ITEM_SIZE.x, (BEATMAPSET_ITEM_SIZE.y + BEATMAP_ITEM_PADDING) * (self.beatmaps.len()+1) as f64)
        }
    }
    fn get_tag(&self) -> String {self.tag.clone()}
    fn set_tag(&mut self, _tag:String) {self.pending_play = false;} // bit of a jank strat: when this is called, reset the play_pending property
    fn get_pos(&self) -> Vector2<f64> {self.pos}
    fn set_pos(&mut self, pos:Vector2<f64>) {self.pos = pos}
    fn get_value(&self) -> Box<dyn std::any::Any> {
        Box::new((self.beatmaps.get(self.selected_item).unwrap().clone(), self.pending_play))
    }

    fn draw(&mut self, _args:RenderArgs, pos_offset:Vector2<f64>) -> Vec<Box<dyn Renderable>> {
        let mut items: Vec<Box<dyn Renderable>> = Vec::new();
        let font = crate::get_font("main");

        let depth = 5.0;

        // draw rectangle
        items.push(Box::new(Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            depth,
            self.pos+pos_offset,
            BEATMAPSET_ITEM_SIZE,
            if self.hover {
                Some(Border {color: Color::RED.into(), radius: 1.0})
            } else if self.selected {
                Some(Border {color: Color::BLUE.into(), radius: 1.0})
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
            format!("{} // {} - {}", self.meta.creator, self.meta.artist, self.meta.title),
            font.clone()
        )));

        // if selected, draw map items
        if self.selected {
            let mut pos = self.pos+pos_offset + Vector2::new(BEATMAPSET_ITEM_SIZE.x - BEATMAP_ITEM_SIZE.x, BEATMAP_ITEM_SIZE.y + BEATMAP_ITEM_PADDING);
            let mut counter = 0;
            
            // try to find the clicked item
            // // we only care about y pos, because we know we were clicked
            let mut index:usize = 999;

            // if x is in correct area to hover over beatmap items
            if self.mouse_pos.x >= self.pos.x + (BEATMAPSET_ITEM_SIZE.x - BEATMAP_ITEM_SIZE.x) {
                let rel_y2 = (self.mouse_pos.y - self.pos.y).abs() - BEATMAPSET_ITEM_SIZE.y;
                index = ((rel_y2 + BEATMAP_ITEM_PADDING/2.0) / (BEATMAP_ITEM_SIZE.y + BEATMAP_ITEM_PADDING)).floor() as usize;
            }

            for b in self.beatmaps.as_slice() {
                // bounding rect
                items.push(Box::new(Rectangle::new(
                    [0.2, 0.2, 0.2, 1.0].into(),
                    depth,
                    pos,
                    BEATMAP_ITEM_SIZE,
                    if counter == index {
                        Some(Border {color: Color::BLUE.into(), radius: 1.0})
                    } else if counter == self.selected_item {
                        Some(Border {color: Color::RED.into(), radius: 1.0})
                    } else {
                        Some(Border {color: Color::BLACK.into(), radius: 1.0})
                    }
                )));
                // version text
                items.push(Box::new(Text::new(
                    Color::WHITE,
                    depth - 1.0,
                    pos + Vector2::new(5.0, 20.0),
                    12,
                    format!("{}", b.lock().unwrap().metadata.version),
                    font.clone()
                )));

                pos.y += BEATMAP_ITEM_SIZE.y + BEATMAP_ITEM_PADDING;
                counter += 1;
            }
        }
        items
    }

    fn on_click(&mut self, pos:Vector2<f64>, _button:MouseButton) -> bool {

        if self.selected && self.hover {
            // find the clicked item

            // we only care about y pos, because we know we were clicked
            let rel_y2 = (pos.y - self.pos.y).abs() - BEATMAPSET_ITEM_SIZE.y;
            let index = (((rel_y2 + BEATMAP_ITEM_PADDING/2.0) / (BEATMAP_ITEM_SIZE.y + BEATMAP_ITEM_PADDING)).floor() as usize).clamp(0, self.beatmaps.len() - 1);

            if self.selected_item == index {
                // queue play map
                self.pending_play = true;
                self.first.lock().unwrap().song.stop();
            } else {
                self.selected_item = index;
            }
            return true;
        }

        // not yet selected
        if !self.selected && self.hover {
            // start song
            self.first.lock().unwrap().song.play();
            self.first.lock().unwrap().song.set_volume(Settings::get().get_music_vol());
        } else { // was selected, not anymore
            // stop music
            self.first.lock().unwrap().song.stop();
        }

        self.selected = self.hover;
        self.hover
    }
    fn on_mouse_move(&mut self, pos:Vector2<f64>) {
        self.mouse_pos = pos;
        self.hover = self.hover(pos)
    }
    fn on_volume_change(&mut self) {
        self.first.lock().unwrap().song.set_volume(Settings::get().get_music_vol());
    }

    fn dispose(&mut self) {
        self.first.lock().unwrap().song.stop();
    }
}


struct LeaderboardItem {
    pos: Vector2<f64>,
    score: Score,

    tag: String,
    hover: bool,

    acc: f64,
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
        items.push(Box::new(Rectangle::new(
            [0.2, 0.2, 0.2, 1.0].into(),
            depth,
            self.pos+pos_offset,
            LEADERBOARD_ITEM_SIZE,
            if self.hover {
                Some(Border {color: Color::RED.into(), radius: 1.0})
            } else {
                None
            }
        )));

        // score text
        items.push(Box::new(Text::new(
            Color::WHITE,
            depth - 1.0,
            self.pos+pos_offset + Vector2::new(5.0, 20.0),
            15,
            format!("{}: {}", self.score.username, crate::format(self.score.score)),
            font.clone()
        )));

        // combo text
        items.push(Box::new(Text::new(
            Color::WHITE,
            depth - 1.0,
            self.pos+pos_offset + Vector2::new(5.0, 40.0),
            12,
            format!("{}x, {:.2}%", crate::format(self.score.max_combo), self.acc),
            font.clone()
        )));

        items
    }

    fn on_click(&mut self, _pos:Vector2<f64>, _button:MouseButton) -> bool {self.hover}
    fn on_mouse_move(&mut self, pos:Vector2<f64>) {self.hover = self.hover(pos);}
}
