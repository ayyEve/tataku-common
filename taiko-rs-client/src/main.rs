// native imports
use std::env;
use std::fmt::Display;
use std::{fs::File, path::Path};
use std::io::{self, BufRead, BufReader, Lines};

// local imports
use game::{Audio, Game, Settings, Vector2};

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
const HIT_AREA_RADIUS:f64 = NOTE_RADIUS * 1.3; // NOTE_RADIUS * 1.3
const HIT_POSITION:Vector2 = Vector2::new(180.0, 200.0);
const PLAYFIELD_RADIUS:f64 = NOTE_RADIUS * 2.0; // actually height, oops
const WINDOW_SIZE:Vector2 = Vector2::new(1000.0, 600.0);


// folders
pub const DOWNLOADS_DIR:&str = "downloads";
pub const SONGS_DIR:&str = "songs";
pub const REPLAYS_DIR:&str = "replays";

// database files
pub const SCORE_DATABASE_FILE:&str = "scores.db";


// helper types
pub type IoError = std::io::Error;

// main fn
fn main() {
    // check for missing folders
    check_folder(DOWNLOADS_DIR, true);
    check_folder(REPLAYS_DIR, true);
    check_folder(SONGS_DIR, true);
    check_folder("fonts", false);
    check_folder("audio", false);

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

// command line settings editing util, not really needed but meh
fn cmd_settings_helper() -> io::Result<()> {
    let mut settings = Settings::get_mut();

    println!("what setting do you want to change?");
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;

    match buffer.trim() {
        "password" => {
            println!("type the pass");
            let mut pass = String::new();
            io::stdin().read_line(&mut pass)?;
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

/// read a file to the end
fn read_lines<P>(filename: P) -> io::Result<Lines<BufReader<File>>> where P: AsRef<Path> {
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
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

