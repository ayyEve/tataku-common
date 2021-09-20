// native imports
use std::fmt::Display;

// local imports
use helpers::io::*;
pub use game::helpers;
pub use ayyeve_piston_ui::render;
pub use ayyeve_piston_ui::render::Vector2;
use game::{Game, Settings, helpers::BenchmarkHelper};

// include files
mod game;
mod menu;
mod errors;
mod gameplay;
mod databases;
mod visualization;

// re-exports to make imports nicer
mod sync {
    pub use std::sync::{Arc, Weak};
    pub use parking_lot::{Mutex, MutexGuard};
}


// folders
pub const DOWNLOADS_DIR:&str = "downloads";
pub const SONGS_DIR:&str = "songs";
pub const REPLAYS_DIR:&str = "replays";

// https://cdn.ayyeve.xyz/taiko-rs/
pub const REQUIRED_FILES:&[&str] = &[
    "resources/audio/don.wav",
    "resources/audio/kat.wav",
    "resources/audio/bigdon.wav",
    "resources/audio/bigkat.wav",
    "resources/audio/combobreak.mp3",
    "resources/audio/slidertick.wav",
    "resources/icon-small.png",
    "fonts/main.ttf",
];

const FIRST_MAPS: &[u32] = &[
    75, // disco prince
    905576, // triumph and regret
    1605148, // mayday
    727903, // galaxy collapse
];

// main fn
#[tokio::main]
async fn main() {
    let mut main_benchmark = BenchmarkHelper::new("main");

    // check for missing folders
    check_folder(DOWNLOADS_DIR);
    check_folder(REPLAYS_DIR);
    check_folder(SONGS_DIR);
    check_folder("fonts");
    check_folder("resources");
    check_folder("resources/audio");

    main_benchmark.log("Folder check done", true);

    // check for missing files
    for file in REQUIRED_FILES.iter() {
        check_file(file, &format!("https://cdn.ayyeve.xyz/taiko-rs/{}", file));
    }


    // hitsounds
    for sample_set in ["normal", "soft", "drum"] {
        for hitsound in ["hitnormal", "hitwhistle", "hitclap", "hitfinish", "slidertick"] {
            let file = &format!("resources/audio/{}-{}.wav", sample_set, hitsound);
            check_file(file, &format!("https://cdn.ayyeve.xyz/taiko-rs/{}", file));
        }
    }

    
    // check if songs folder is empty
    if std::fs::read_dir(SONGS_DIR).unwrap().count() == 0 {
        // no songs, download some
        for id in FIRST_MAPS {
            check_file(&format!("{}/{}.osz", DOWNLOADS_DIR, id), &format!("https://cdn.ayyeve.xyz/taiko-rs/maps/{}.osz", id));
        }
    }


    main_benchmark.log("File check done", true);
    
    let game = Game::new();
    main_benchmark.log("Game creation complete", true);

    drop(main_benchmark);
    game.game_loop();
}


// helper functions


/// format a number into a locale string ie 1000000 -> 1,000,000
fn format<T:Display>(num:T) -> String {
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
