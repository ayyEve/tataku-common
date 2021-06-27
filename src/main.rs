use std::env;
// native imports
use std::fmt::Display;
use std::sync::{Arc, Mutex};
use std::{fs::File, path::Path};
use std::collections::hash_map::HashMap;
use std::io::{self, BufRead, BufReader, Lines, Read, Write};

// crate imports
use cgmath::Vector2;
use opengl_graphics::{GlyphCache, TextureSettings};

// local imports
use game::{Audio, Game, SerializationReader, SerializationWriter};

use crate::game::Settings;

// include files
mod game;
mod gameplay;
mod render;
mod menu;
mod databases;
mod enums;
pub use enums::*;

// constants
const NOTE_RADIUS:f64 = 32.0;
const HIT_AREA_RADIUS:f64 = NOTE_RADIUS * 1.3;
const HIT_POSITION:Vector2<f64> = Vector2::new(130.0, 200.0);
const PLAYFIELD_RADIUS:f64 = NOTE_RADIUS * 2.0; // actually height, oops
const WINDOW_SIZE:Vector2<u32> = Vector2::new(1000, 600);


// folders
pub const DOWNLOADS_DIR:&str = "downloads";
pub const SONGS_DIR:&str = "songs";


// font stuff
const FONT_LIST:[&'static str; 1] = [
    "fonts/main.ttf"
];
lazy_static::lazy_static! {
    pub static ref FONTS: HashMap<String, Arc<Mutex<GlyphCache<'static>>>> = {
        let mut fonts:HashMap<String, Arc<Mutex<GlyphCache<'static>>>> = HashMap::new();
        for font in FONT_LIST.iter() {
            let font_name = Path::new(font).file_stem().unwrap().to_str().unwrap();
            let glyphs = GlyphCache::new(font, (), TextureSettings::new()).unwrap();
            fonts.insert(font_name.to_owned(), Arc::new(Mutex::new(glyphs)));
        }

        fonts
    };
}


fn main() {
    // check for missing folders
    check_folder(DOWNLOADS_DIR, true);
    check_folder(SONGS_DIR, true);
    check_folder("fonts", true);
    check_folder("audio", true);

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        if args[1].starts_with("settings") {
            cmd_settings_helper().expect("Error: ");
        }
        return;
    }

    
    // intialize audio engine
    let _stream = Audio::setup();

    let game = Game::new();
    game.game_loop();
}


// helper functions
fn check_folder(dir:&str, create:bool) {
    if !Path::new(dir).exists() {
        if create {
            std::fs::create_dir(dir).expect("error creating folder: ");
        } else {
            panic!("folder does not exist, but is required: {}", dir);
        }
    }
}

// command line settings editing util, mainly because copy/paste doesnt work lol
fn cmd_settings_helper() -> io::Result<()> {
    let mut settings = Settings::get_mut();

    println!("what setting do you want to change?");
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;

    match buffer.trim() {
        "password" => {
            println!("type the pass");
            let mut pass = String::new();
            io::stdin().read_to_string(&mut pass)?;
            settings.password = pass.trim().to_owned();
            settings.save();
        }

        "close" => {
            return std::io::Result::Ok(());
        }

        _ => {
            println!("unknown property");
        }
    }
    std::io::Result::Ok(())
}

/// get a font, or `main` if font is not found
fn get_font(name:&str) -> Arc<Mutex<GlyphCache<'static>>>{
    if FONTS.contains_key(name) {
        return FONTS.get(name).unwrap().clone();
    }

    println!("[FONT] > attempted to load non-existing font \"{}\"", name);
    FONTS.get("main").unwrap().clone()
}

/// read a file to the end
fn read_lines<P>(filename: P) -> io::Result<Lines<BufReader<File>>> where P: AsRef<Path> {
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
}

pub fn open_database(filename:&str) -> std::io::Result<SerializationReader> {
    let file = File::open(filename)?; //.expect(&format!("Error opening database file {}", filename));
    let mut buf:Vec<u8> = Vec::new();
    BufReader::new(file).read_to_end(&mut buf)?; //.expect("error reading database file");
    Ok(SerializationReader::new(buf))
}

pub fn save_database(filename:&str, writer:SerializationWriter) -> std::io::Result<()> {
    let bytes = writer.data();
    let bytes = bytes.as_slice();
    let mut f = File::create(filename)?;
    f.write_all(bytes)?;
    f.flush()?;
    Ok(())
}

/// format a number into a locale string ie 1000000 -> 1,000,000
fn format<T>(num:T) -> String where T:Display{
    let str = format!("{}", num);
    let mut split = str.split(".");
    let num = split.next().unwrap();
    let dec = split.next();

    // split num into 3s
    let mut new_str = String::new();
    let offset = num.len() % 3;

    num.char_indices().rev().for_each(|(pos, char)| {
        new_str.push(char);
        if pos % 3 == offset {
            new_str.push(',');
        }
    });
    if dec.is_some() {
        new_str += &format!(".{}", dec.unwrap());
    }

    let mut new_new = String::with_capacity(new_str.len());
    new_new.extend(new_str.chars().rev());
    new_new.trim_start_matches(",").to_owned()
}

