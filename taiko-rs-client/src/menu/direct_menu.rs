use std::collections::HashMap;
use std::{fs::File, io::Write};

use piston::{Key, MouseButton};

use crate::sync::Arc;
use crate::{window_size, DOWNLOADS_DIR, Vector2};
use crate::render::{Text, Renderable, Rectangle, Color, Border};
use crate::menu::{Menu, ScrollableArea, ScrollableItem, TextInput};
use crate::game::{Audio, Game, GameState, KeyModifiers, Settings, get_font};

const DOWNLOAD_ITEM_SIZE:Vector2 = Vector2::new(300.0, 40.0);
const DOWNLOAD_ITEM_YMARGIN:f64 = 30.0;
const DOWNLOAD_ITEM_YOFFSET:f64 = SEARCH_BAR_HEIGHT + 10.0;
const DOWNLOAD_ITEM_XOFFSET:f64 = 5.0;
const SEARCH_BAR_HEIGHT:f64 = 50.0;
//TODO: change this to its own manager or smth
const DIRECT_ITEM_SIZE:Vector2 = Vector2::new(500.0, 80.0);

//TODO: properly implement this lol
const MAX_CONCURRENT_DOWNLOADS:usize = 5;

pub struct OsuDirectMenu {
    scroll_area: ScrollableArea,

    items: HashMap<String, Arc<DirectItem>>,
    downloading: Vec<Arc<DirectItem>>,

    queue: Vec<Arc<DirectItem>>,
    selected: Option<String>,

    /// attempted? succeeded? (path, pos)
    old_audio: Option<Option<(String, f32)>>,

    search_bar: TextInput
}
impl OsuDirectMenu {
    pub fn new() -> OsuDirectMenu {
        let mut x = OsuDirectMenu {
            scroll_area: ScrollableArea::new(Vector2::new(0.0, SEARCH_BAR_HEIGHT+5.0), Vector2::new(DIRECT_ITEM_SIZE.x, window_size().y - SEARCH_BAR_HEIGHT+5.0), true),
            downloading: Vec::new(),
            queue: Vec::new(),
            items: HashMap::new(),
            selected: None,
            search_bar: TextInput::new(Vector2::zero(), Vector2::new(window_size().x , SEARCH_BAR_HEIGHT), "Search", ""),
            old_audio: None
        };
        // TODO: [audio] pause playing music, store song and pos. on close, put it back how it was

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

        let req = reqwest::blocking::get(url.clone());
        if let Ok(thing) = req {
            let data = thing.bytes().expect("error converting mp3 preview to bytes");
            let mut data2 = Vec::new();
            data.iter().for_each(|e| data2.push(e.clone()));

            // store last playing audio if needed
            if self.old_audio.is_none() {
                if let Some((key, a)) = Audio::get_song_raw() {
                    if let Some(a2) = a.upgrade() {
                        self.old_audio = Some(Some((key, a2.current_time())));
                    }
                }
                // need to store that we made an attempt
                if let None = self.old_audio {
                    self.old_audio = Some(None);
                }
            }

            Audio::play_song_raw(url, data2);
        } else if let Err(oof) = req {
            println!("error with preview: {}", oof);
        }
    }

    /// go back to the main menu
    fn back(&mut self, game:&mut Game) {

        if let Some(old_audio) = &self.old_audio {
            Audio::stop_song();

            // restore previous audio
            if let Some((path, pos)) = old_audio.clone() {
                Audio::play_song(path, false, pos);
            }
        }

        let menu = game.menus.get("main").unwrap().clone();
        game.queue_state_change(GameState::InMenu(menu));
    }
}
impl Menu<Game> for OsuDirectMenu {
    fn update(&mut self, _game:&mut Game) {
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
        list.extend(self.scroll_area.draw(args, Vector2::zero(), 0.0));

        list.extend(self.search_bar.draw(args, Vector2::zero(), -90.0));

        // draw download items
        if self.downloading.len() > 0 {

            let x = args.window_size[0] - (DOWNLOAD_ITEM_SIZE.x+DOWNLOAD_ITEM_XOFFSET);

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
    
    fn on_scroll(&mut self, delta:f64, _game:&mut Game) {
        self.scroll_area.on_scroll(delta);
    }

    fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, _game:&mut Game) {
        self.search_bar.on_click(pos, button, mods);

        if let Some(key) = self.scroll_area.on_click_tagged(pos, button, mods) {
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

    fn on_mouse_move(&mut self, pos:Vector2, _game:&mut Game) {
        self.search_bar.on_mouse_move(pos);
        self.scroll_area.on_mouse_move(pos);
    }

    fn on_key_press(&mut self, key:Key, game:&mut Game, mods:KeyModifiers) {
        self.search_bar.on_key_press(key, mods);

        if key == Key::Escape {return self.back(game)}

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
            pos: Vector2::zero(), // being set by the scroll area anyways
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
        
        let username = Settings::get().username;
        let password = Settings::get().password;
        let url = format!("https://osu.ppy.sh/d/{}?u={}&h={:x}", self.item.filename.clone(), username, md5::compute(password));

        perform_download(url, download_dir);
    }
}
impl ScrollableItem for DirectItem {
    // fn update(&mut self) {}
    fn size(&self) -> Vector2 {DIRECT_ITEM_SIZE}
    fn set_pos(&mut self, pos:Vector2) {self.pos = pos}
    fn get_pos(&self) -> Vector2 {self.pos}
    fn get_tag(&self) -> String {self.item.filename.clone()}
    fn set_tag(&mut self, _tag:&str) {}

    fn get_hover(&self) -> bool {self.hover}
    fn set_hover(&mut self, hover:bool) {self.hover = hover}
    fn get_selected(&self) -> bool {self.selected}
    fn set_selected(&mut self, selected:bool) {self.selected = selected}

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

    fn on_click(&mut self, _pos:Vector2, _button:piston::MouseButton, _mods:KeyModifiers) -> bool {
        if self.selected && self.hover {self.download()}

        self.selected = self.hover;
        self.hover
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
