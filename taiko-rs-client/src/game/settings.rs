use std::sync::Arc;

use piston::Key;
use parking_lot::{Mutex, MutexGuard};

use crate::Vector2;
use taiko_rs_common::serialization::*;

const SETTINGS_DATABASE_FILE:&str = "settings.db";
const SETTINGS_VERSION:u32 = 3;

lazy_static::lazy_static! {
    static ref SETTINGS: Arc<Mutex<Settings>> = Arc::new(Mutex::new(Settings::load()));
}


#[derive(Clone, Debug)]
pub struct Settings {
    // volume
    pub master_vol: f32,
    pub music_vol: f32,
    pub effect_vol: f32,
    
    // osu
    pub username: String,
    pub password: String,

    // sb
    pub static_sv: bool,
    pub sv_multiplier: f32,

    // keys
    pub left_kat: Key,
    pub left_don: Key,
    pub right_don: Key,
    pub right_kat: Key,

    // window settings
    pub unlimited_fps: bool,
    pub fps_target: u64,
    pub update_target: u64,

    // bg
    pub background_dim: f32,

    // use this later
    pub window_size: Vector2,
}
impl Settings {
    fn load() -> Settings {
        match open_database(SETTINGS_DATABASE_FILE) {
            Ok(mut reader) => reader.read(),
            Err(e) => {
                println!("Error reading db: {:?}", e);
                Default::default()
            },
        }
    }
    pub fn save(&self) {
        println!("Saving settings");
        let mut writer = SerializationWriter::new();
        writer.write(self.clone());
        
        // write file
        save_database(SETTINGS_DATABASE_FILE, writer).expect("Error saving settings.");
    }

    // relatively slow, if you need a more performant get, use get_mut
    pub fn get() -> Settings {SETTINGS.lock().clone()}
    pub fn get_mut<'a>() -> MutexGuard<'a, Settings> {SETTINGS.lock()}

    pub fn get_effect_vol(&self) -> f32 {self.effect_vol * self.master_vol}
    pub fn get_music_vol(&self) -> f32 {self.music_vol * self.master_vol}
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            // vol
            music_vol: 1.0,
            effect_vol: 1.0,
            master_vol: 0.3,

            // osu
            username: "Guest".to_owned(),
            password: "".to_owned(),

            // keys
            left_kat: Key::D,
            left_don: Key::F,
            right_don: Key::J,
            right_kat: Key::K,

            // sv
            static_sv: false,
            sv_multiplier: 1.0,

            // window settings
            unlimited_fps: false,
            fps_target: 144,
            update_target: 1000,
            window_size: Vector2::new(1000.0, 600.0),
            background_dim: 0.8
        }
    }
}

impl Serializable for Settings {
    fn read(sr:&mut SerializationReader) -> Self {
        let version:u32 = sr.read();

        let mut s = Settings {
            master_vol: sr.read(),
            effect_vol: sr.read(),
            music_vol: sr.read(),
            username: sr.read(),
            password: sr.read(),

            left_kat: sr.read_u32().into(),
            left_don: sr.read_u32().into(),
            right_don: sr.read_u32().into(),
            right_kat: sr.read_u32().into(),

            ..Default::default()
        };

        if version > 1 { // 2 and above
            s.static_sv = sr.read();
            s.sv_multiplier = sr.read();
        }
        if version > 2 { // 3 and above
            s.unlimited_fps = sr.read();
            s.fps_target = sr.read();
            s.update_target = sr.read();
            s.window_size = Vector2::new(
                sr.read(),
                sr.read()
            );
            s.background_dim = sr.read();
        }

        s
    }

    fn write(&self, sw:&mut SerializationWriter) {
        sw.write(SETTINGS_VERSION);
        // volume
        sw.write(self.master_vol);
        sw.write(self.effect_vol);
        sw.write(self.music_vol);
        
        // osu
        sw.write(self.username.clone());
        sw.write(self.password.clone());

        // keys
        sw.write(self.left_kat as u32);
        sw.write(self.left_don as u32);
        sw.write(self.right_don as u32);
        sw.write(self.right_kat as u32);
        
        // v2 and above
        sw.write(self.static_sv);
        sw.write(self.sv_multiplier);

        // v3 and above
        sw.write(self.unlimited_fps);
        sw.write(self.fps_target);
        sw.write(self.update_target);
        sw.write(self.window_size.x);
        sw.write(self.window_size.y);
        sw.write(self.background_dim);
    }
}
