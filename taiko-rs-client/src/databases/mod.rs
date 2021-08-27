use std::sync::Arc;
use parking_lot::Mutex;
use rusqlite::Connection;

mod score_database;
pub use score_database::*;





lazy_static::lazy_static! {
    pub static ref DATABASE: Arc<Mutex<Connection>> = {
        let db = Connection::open("taiko-rs.db").unwrap();
        
        // scores table
        db.execute(
            // pub username: String,
            // pub beatmap_hash: String,
            // pub playmode: PlayMode,
            // pub score: u64,
            // pub combo: u16,
            // pub max_combo: u16,
            // pub x100: u16,
            // pub x300: u16,
            // pub xmiss: u16,
            "CREATE TABLE IF NOT EXISTS scores (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT,
                map_hash TEXT,
                playmode INTEGER,
                score INTEGER,
                combo INTEGER,
                max_combo INTEGER,
                x100 INTEGER,
                x300 INTEGER,
                xmiss INTEGER
         )", [])
        .expect("error creating db table");

        // beatmaps table
        db.execute(
            // pub file_path: String,
            // pub beatmap_hash: String,
            // pub mode: PlayMode,
            // pub beatmap_version: f32,
            // pub artist: String,
            // pub title: String,
            // pub artist_unicode: String,
            // pub title_unicode: String,
            // pub creator: String,
            // pub version: String,

            // pub audio_filename: String,
            // pub image_filename: String,
            // pub audio_preview: f32,

            // pub duration: u64,
            // mins: u8,
            // secs: u8,

            // pub hp: f32,
            // pub od: f32,
            // pub sr: f64,
            // pub slider_multiplier: f32,
            // pub slider_tick_rate: f32,
            "CREATE TABLE IF NOT EXISTS beatmaps (
                beatmap_path TEXT,
                beatmap_hash TEXT,

                playmode INTEGER,
                beatmap_version INTEGER,
                artist TEXT,
                title TEXT,
                artist_unicode TEXT,
                title_unicode TEXT,
                creator TEXT,
                version TEXT,

                audio_filename TEXT,
                image_filename TEXT,
                audio_preview REAL,
                
                duration REAL,
                
                hp REAL,
                od REAL,
                cs REAL,
                ar REAL,
                
                slider_multiplier REAL,
                slider_tick_rate REAL
            )", [])
        .expect("error creating db table");

        Arc::new(Mutex::new(db))
    };
    
}

