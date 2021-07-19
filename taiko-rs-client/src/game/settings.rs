use std::sync::Arc;

use parking_lot::{Mutex, MutexGuard};
use piston::Key;

use taiko_rs_common::serialization::*;

const SETTINGS_DATABASE_FILE:&str = "settings.db";
const SETTINGS_VERSION: u32 = 2;

lazy_static::lazy_static! {
    static ref SETTINGS: Arc<Mutex<Settings>> = {
        Arc::new(Mutex::new(Settings::load()))
    };
}


#[derive(Clone, Debug)]
pub struct Settings {
    pub master_vol: f32,
    pub music_vol: f32,
    pub effect_vol: f32,
    pub username: String,
    pub password: String,

    pub static_sv: bool,
    pub sv_multiplier: f32,

    // keys
    pub left_kat: Key,
    pub left_don: Key,
    pub right_don: Key,
    pub right_kat: Key
}
impl Settings {
    fn load() -> Settings {
        let reader = open_database(SETTINGS_DATABASE_FILE);
        match reader {
            Err(e) => {
                println!("Error reading db: {:?}", e);
                Settings {
                    music_vol: 0.3,
                    effect_vol: 0.3,
                    master_vol: 1.0,
                    username: "Guest".to_owned(),
                    password: "".to_owned(),

                    left_kat: Key::D,
                    left_don: Key::F,
                    right_don: Key::J,
                    right_kat: Key::K,

                    static_sv: false,
                    sv_multiplier: 1.0
                }
            },
            Ok(mut reader) => {
                reader.read::<Settings>()
            }
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
    pub fn get() -> Settings {
        SETTINGS.lock().clone()
    }
    pub fn get_mut<'a>() -> MutexGuard<'a, Settings> {
        SETTINGS.lock()
    }

    pub fn get_effect_vol(&self) -> f32 {
        self.effect_vol * self.master_vol
    }
    pub fn get_music_vol(&self) -> f32 {
        self.music_vol * self.master_vol
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

            static_sv: false,
            sv_multiplier: 1.0
        };

        if version > 1 { // 2 and above
            s.static_sv = sr.read();
            s.sv_multiplier = sr.read();
        }

        s
    }

    fn write(&self, sw:&mut SerializationWriter) {
        sw.write(SETTINGS_VERSION);
        sw.write(self.master_vol);
        sw.write(self.effect_vol);
        sw.write(self.music_vol);
        sw.write(self.username.clone());
        sw.write(self.password.clone());

        sw.write(self.left_kat as u32);
        sw.write(self.left_don as u32);
        sw.write(self.right_don as u32);
        sw.write(self.right_kat as u32);
        
        // v2 and above
        sw.write(self.static_sv);
        sw.write(self.sv_multiplier);
    }
}
