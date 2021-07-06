use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::{fs::File, io::Write};

use piston::{Key, MouseButton};
use rodio::Sink; // ugh

use crate::{WINDOW_SIZE, DOWNLOADS_DIR};
use crate::render::{Text, Renderable, Rectangle, Color, Border};
use crate::menu::{Menu, ScrollableArea, ScrollableItem, TextInput};
use crate::game::{Audio, Game, GameMode, KeyModifiers, Settings, get_font, Vector2};

const DIRECT_ITEM_SIZE:Vector2 = Vector2::new(600.0, 80.0);
const SEARCH_BAR_HEIGHT:f64 = 50.0;
const DOWNLOAD_ITEM_SIZE:Vector2 = Vector2::new(300.0, 40.0);
const DOWNLOAD_ITEM_YMARGIN:f64 = 30.0;
const DOWNLOAD_ITEM_YOFFSET:f64 = SEARCH_BAR_HEIGHT + 10.0;

//TODO: properly implement this lol
const MAX_CONCURRENT_DOWNLOADS:usize = 5;

pub struct OsuDirectMenu {
    scroll_area: ScrollableArea,

    items: HashMap<String, Arc<DirectItem>>,
    downloading: Vec<Arc<DirectItem>>,

    queue: Vec<Arc<DirectItem>>,
    selected: Option<String>,

    search_bar: TextInput,

    //TODO: figure out how to get this running in a separate thread
    preview: Option<Arc<Mutex<Sink>>>
}
impl OsuDirectMenu {
    pub fn new() -> OsuDirectMenu {
        let mut x = OsuDirectMenu {
            scroll_area: ScrollableArea::new(Vector2::new(0.0, SEARCH_BAR_HEIGHT+5.0), Vector2::new(WINDOW_SIZE.x as f64, WINDOW_SIZE.y as f64 - (SEARCH_BAR_HEIGHT+5.0)), true),
            downloading: Vec::new(),
            queue: Vec::new(),
            items: HashMap::new(),
            selected: None,
            preview: None,
            search_bar: TextInput::new(Vector2::new(0.0, 0.0), Vector2::new(WINDOW_SIZE.x as f64, SEARCH_BAR_HEIGHT), "Search", "")
        };
        x.do_search();
        x
    }
    fn do_search(&mut self) {
        let q = self.search_bar.get_text();
        let settings = Settings::get();

        let data = do_search(settings.username, settings.password, 1, 0, 0, if q.len() > 0 {Some(q)} else {None});
        let mut lines = data.split('\n');
        let count = lines.next().unwrap_or("0").parse::<i32>().unwrap_or(0);
        if count <= 0 {return}
        println!("got {} items", count);
        self.scroll_area.clear();
        self.items.clear();

        for line in lines {
            if line.len() < 5 {continue}
            let i = DirectItem::from_str(line.to_owned());
            let a = Arc::new(i.clone());
            self.items.insert(i.get_tag(), a);
            self.scroll_area.add_item(Box::new(i));
        }
    }

    fn do_preview_audio(&mut self, set_id:String) {
        println!("preview audio");

        // https://b.ppy.sh/preview/100.mp3
        let url = format!("https://b.ppy.sh/preview/{}.mp3", set_id);
        if let Some(sink) = &self.preview {sink.lock().unwrap().stop()}

        let req = reqwest::blocking::get(url);
        if let Ok(thing) = req {
            let data = thing.bytes().expect("error converting mp3 preview to bytes");
            let mut data2 = Vec::new();
            data.iter().for_each(|e| data2.push(e.clone()));

            let sink = Audio::from_raw(data2);
            sink.set_volume(Settings::get().get_music_vol());
            sink.play();
            self.preview = Some(Arc::new(Mutex::new(sink)));
        } else if let Err(oof) = req {
            println!("error with preview: {}", oof);
        }        
    }
}
impl Menu for OsuDirectMenu {
    fn update(&mut self, _game:Arc<Mutex<&mut Game>>) {
        // check download statuses
        let dir = std::fs::read_dir(DOWNLOADS_DIR).unwrap();
        let mut files = Vec::new();
        dir.for_each(|f|{
            if let Ok(thing) = f {
                files.push(thing.file_name().to_string_lossy().to_string());
            }
        });

        //TODO: maybe just read the downloads dir and see if the file is contained there
        self.downloading.retain(|i| {
            !files.contains(&i.item.filename)
        });

        while self.downloading.len() < MAX_CONCURRENT_DOWNLOADS && self.queue.len() > 0 {
            let i = self.queue.remove(0);
            self.downloading.push(i)
        }
    }

    fn draw(&mut self, args:piston::RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list:Vec<Box<dyn Renderable>> = Vec::new();
        list.extend(self.scroll_area.draw(args));

        list.extend(self.search_bar.draw(args, Vector2::new(0.0,0.0), -90.0));

        // draw download items
        if self.downloading.len() > 0 {

            let x = args.window_size[0] - (DOWNLOAD_ITEM_SIZE.x+5.0);

            list.push(Box::new(Rectangle::new(
                Color::WHITE,
                3.0,
                Vector2::new(x, DOWNLOAD_ITEM_YOFFSET),
                Vector2::new(DOWNLOAD_ITEM_SIZE.x, args.window_size[1] - DOWNLOAD_ITEM_YOFFSET*2.0),
                Some(Border::new(Color::BLACK, 1.8))
            )));
            
            let mut counter = 0.0;
            let font = get_font("main");

            // downloading
            for i in self.downloading.as_slice() {
                let pos = Vector2::new(x, DOWNLOAD_ITEM_YOFFSET + (DOWNLOAD_ITEM_SIZE.y + DOWNLOAD_ITEM_YMARGIN) * counter);
                // bounding box
                list.push(Box::new(Rectangle::new(
                    Color::WHITE,
                    2.0,
                    pos,
                    DOWNLOAD_ITEM_SIZE,
                    Some(Border::new(Color::BLUE, 1.5))
                )));
                // map text
                list.push(Box::new(Text::new(
                    Color::BLACK,
                    1.0,
                    pos + Vector2::new(0.0, 15.0),
                    15,
                    format!("{} (Downloading)", i.item.title),
                    font.clone()
                )));

                counter += 1.0;
            }
            
            // queued
            for i in self.queue.as_slice() {
                let pos = Vector2::new(x, DOWNLOAD_ITEM_YOFFSET + (DOWNLOAD_ITEM_SIZE.y + DOWNLOAD_ITEM_YMARGIN) * counter);
                // bounding box
                list.push(Box::new(Rectangle::new(
                    Color::WHITE,
                    2.0,
                    pos,
                    DOWNLOAD_ITEM_SIZE,
                    Some(Border::new(Color::BLACK, 1.5))
                )));
                // map text
                list.push(Box::new(Text::new(
                    Color::BLACK,
                    1.0,
                    pos + Vector2::new(0.0, 15.0),
                    15,
                    format!("{} (Waiting...)", i.item.title),
                    font.clone()
                )));

                counter += 1.0;
            }
        }

        list
    }
    
    fn on_scroll(&mut self, delta:f64, _game:Arc<Mutex<&mut Game>>) {
        self.scroll_area.on_scroll(delta);
    }

    fn on_click(&mut self, pos:Vector2, button:MouseButton, game:Arc<Mutex<&mut Game>>) {
        self.search_bar.on_click(pos, button);

        if let Some(key) = self.scroll_area.on_click(pos, button, game) {
            if let Some(selected) = self.selected.clone() {
                if key == selected {
                    if let Some(item) = self.items.get(&key) {
                        if item.downloading {return};
                        self.queue.push(item.clone());
                    }
                    return;
                }
            }

            if let Some(item) = self.items.clone().get(&key) {
               self.do_preview_audio(item.item.set_id.clone());
            }
            self.selected = Some(key.clone());
        }
    }

    fn on_mouse_move(&mut self, pos:Vector2, game:Arc<Mutex<&mut Game>>) {
        self.search_bar.on_mouse_move(pos);
        self.scroll_area.on_mouse_move(pos, game);
    }

    fn on_key_press(&mut self, key:Key, game:Arc<Mutex<&mut Game>>, mods:KeyModifiers) {
        self.search_bar.on_key_press(key, mods);

        if key == Key::Escape {
            let mut game = game.lock().unwrap();
            game.beatmap_pending_refresh = true;
            let menu = game.menus.get("main").unwrap().clone();
            game.queue_mode_change(GameMode::InMenu(menu));
        }

        println!("got key {:?}", key);
        if key == Key::Return {
            self.do_search();
        }
        
    }

    fn on_text(&mut self, text:String) {
        self.search_bar.on_text(text);
    }
}


fn do_search(username:String, password:String, mode:u8, page:u8, sort:u8, query:Option<String>) -> String {
    println!("doing search");
    let password:String = format!("{:x}", md5::compute(password));

    let mut url = format!("https://osu.ppy.sh/web/osu-search.php?u={}&h={}&m={}&p={}&s={}", username, password, mode, page, sort);
    if let Some(q) = query {
        url += format!("&q={}", q).as_str();
    }

    // url = "https://osu.ppy.sh/web/osu-search.php?u=emibot&h=cdcda2be9c41247386eaa646edb132c2".to_owned();
    let body = reqwest::blocking::get(url)
        .expect("error with request")
        .text()
        .expect("error converting to text");

    body
}

/// perform a download on another thread
fn perform_download(url:String, path:String) {
    std::thread::spawn(move || {
        let file_bytes = reqwest::blocking::get(url)
            .expect("error with request")
            .bytes()
            .expect("error getting bytes");
            
        write_file(path, &file_bytes).expect("Error writing file");
    });
}

fn write_file(file:String, bytes:&[u8]) -> std::io::Result<()> {
    let mut f = File::create(file)?;
    f.write_all(bytes)?;
    f.flush()?;

    Ok(())
}

#[derive(Clone)]
struct DirectItem {
    pos: Vector2,

    item: DirectMeta,
    hover: bool,
    selected: bool,
    pub downloading: bool,
}
impl DirectItem {
    pub fn from_str(str:String) -> DirectItem {
        DirectItem {
            pos: Vector2::new(0.0, 0.0), // being set by the scroll area anyways
            item: DirectMeta::from_str(str.clone()),

            hover: false,
            selected:false,
            downloading: false
        }   
    }

    pub fn download(&mut self) {
        if self.downloading {return}
        self.downloading = true;
        let download_dir = format!("downloads/{}", self.item.filename.clone());
        
        // let username = Settings::get().username;
        // let password = Settings::get().password;
        // let url = format!("https://osu.ppy.sh/d/{}?u={}&h={:x}", self.item.filename.clone(), username, md5::compute(password));

        let url = format!("https://osu.ppy.sh/d/{}?u=emibot&h=cdcda2be9c41247386eaa646edb132c2", self.item.filename.clone());

        perform_download(url, download_dir);
    }
}
impl ScrollableItem for DirectItem {
    // fn update(&mut self) {}
    fn size(&self) -> Vector2 {DIRECT_ITEM_SIZE}
    fn set_pos(&mut self, pos:Vector2) {self.pos = pos;}
    fn get_pos(&self) -> Vector2 {self.pos}
    fn get_tag(&self) -> String {self.item.filename.clone()}
    fn set_tag(&mut self, _tag:&str) {}

    fn draw(&mut self, _args:piston::RenderArgs, pos_offset:Vector2, parent_depth:f64) -> Vec<Box<dyn Renderable>> {
        let mut list:Vec<Box<dyn Renderable>> = Vec::new();
        let font = get_font("main");

        list.push(Box::new(Rectangle::new(
            Color::WHITE,
            parent_depth + 10.0,
            self.pos + pos_offset,
            self.size(),
            Some(Border::new(if self.hover {Color::BLUE} else if self.selected {Color::RED} else {Color::BLACK}, 1.5))
        )));

        list.push(Box::new(Text::new(
            Color::BLACK,
            parent_depth + 9.9,
            self.pos+Vector2::new(5.0, 25.0) + pos_offset,
            20,
            format!("{} - {}", self.item.artist, self.item.title),
            font.clone()
        )));

        list.push(Box::new(Text::new(
            Color::BLACK,
            parent_depth + 9.9,
            self.pos+Vector2::new(5.0, 50.0) + pos_offset,
            20,
            format!("Mapped by {}", self.item.creator),
            font.clone()
        )));

        list
    }

    fn on_click(&mut self, _pos:Vector2, _button:piston::MouseButton) -> bool {

        if self.selected {
            if self.hover {
                self.download();
            } else {
                self.selected  = false;
                return false;
            }
        }

        if self.hover {
            self.selected = true;
            return true;
        }

        false
    }

    fn on_mouse_move(&mut self, pos:Vector2) {
        self.hover = self.hover(pos);
    }
}


// TODO: figure out how to get the progress of the download
#[derive(Clone)]
struct DirectMeta {
    set_id: String,
    filename: String,
    artist: String,
    title: String,
    creator: String,
}
impl DirectMeta {
    pub fn from_str(str:String) -> DirectMeta {
        // println!("reading {}", str);
        let mut split = str.split('|');

        // 867737.osz|The Quick Brown Fox|The Big Black|Mismagius|1|9.37143|2021-06-25T02:25:11+00:00|867737|820065|||0||Easy ★1.9@0,Normal ★2.5@0,Advanced ★3.2@0,Hard ★3.6@0,Insane ★4.8@0,Extra ★5.6@0,Extreme ★6.6@0,Remastered Extreme ★6.9@0,Riddle me this riddle me that... ★7.5@0
        // filename, artist, title, creator, ranking_status, rating, last_update, beatmapset_id, thread_id, video, storyboard, filesize, filesize_novideo||filesize, difficulty_names

        let filename = split.next().expect("[Direct] err:filename").to_owned();
        let artist = split.next().expect("[Direct] err:artist").to_owned();
        let title = split.next().expect("[Direct] err:title").to_owned();
        let creator = split.next().expect("[Direct] err:creator").to_owned();
        let _ranking_status = split.next();
        let _rating = split.next();
        let _last_update = split.next();
        let beatmapset_id = split.next().unwrap();

        DirectMeta {
            set_id: beatmapset_id.to_owned(),
            filename,
            artist,
            title,
            creator
        }
    }
}
