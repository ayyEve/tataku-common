// native imports
use std::fmt::Display;
use std::{fs::File, path::Path};
use std::io::{self, BufRead, BufReader, Lines};

// local imports
use game::{Game, helpers::BenchmarkHelper};
pub use ayyeve_piston_ui::render;
pub use ayyeve_piston_ui::render::Vector2;

// include files
mod game;
mod gameplay;
mod menu;
mod databases;
pub use game::helpers;

// constants
const WINDOW_SIZE:Vector2 = Vector2::new(1000.0, 600.0);

// folders
pub const DOWNLOADS_DIR:&str = "downloads";
pub const SONGS_DIR:&str = "songs";
pub const REPLAYS_DIR:&str = "replays";

// database files
pub const SCORE_DATABASE_FILE:&str = "scores.db";

// main fn
fn main() {
    let mut main_benchmark = BenchmarkHelper::new("main");

    // check for missing folders
    check_folder(DOWNLOADS_DIR, true);
    check_folder(REPLAYS_DIR, true);
    check_folder(SONGS_DIR, true);
    check_folder("fonts", false);
    check_folder("audio", false);
    main_benchmark.log("folder check done", true);
    
    let game = Game::new();
    let _ = game.threading.enter();
    main_benchmark.log("game creation complete", true);

    drop(main_benchmark);
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
