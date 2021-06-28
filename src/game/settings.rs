use std::sync::{Arc, Mutex, MutexGuard};
use piston::Key;

use crate::game::helpers::{SerializationReader, SerializationWriter, Serializable, open_database, save_database};

const SETTINGS_DATABASE_FILE:&str = "settings.db";
const SETTINGS_VERSION: u32 = 1;

lazy_static::lazy_static! {
    static ref SETTINGS: Arc<Mutex<Settings>> = {
        let settings = Settings::load();
        Arc::new(Mutex::new(settings))
    };
}


#[derive(Clone, Debug)]
pub struct Settings {
    pub master_vol: f32,
    pub music_vol: f32,
    pub effect_vol: f32,
    pub username: String,
    pub password: String,

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
                    right_kat: Key::K
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

    pub fn get() -> Settings {
        SETTINGS.lock().unwrap().clone()
    }
    pub fn get_mut<'a>() -> MutexGuard<'a, Settings> {
        SETTINGS.lock().unwrap()
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
        let _version:u32 = sr.read();

        Settings {
            master_vol: sr.read(),
            effect_vol: sr.read(),
            music_vol: sr.read(),
            username: sr.read(),
            password: sr.read(),

            left_kat: sr.read(),
            left_don: sr.read(),
            right_don: sr.read(),
            right_kat: sr.read()
        }
    }

    fn write(&self, sw:&mut SerializationWriter) {
        sw.write(SETTINGS_VERSION);
        sw.write(self.master_vol);
        sw.write(self.effect_vol);
        sw.write(self.music_vol);
        sw.write(self.username.clone());
        sw.write(self.password.clone());

        sw.write(self.left_kat);
        sw.write(self.left_don);
        sw.write(self.right_don);
        sw.write(self.right_kat);
    }
}

// allows the serialization of keys
impl Serializable for Key {
    fn read(sr:&mut SerializationReader) -> Self {
        sr.read_u32().into()
    }

    fn write(&self, sw:&mut SerializationWriter) {
        sw.write(*self as u32);
    }
}
