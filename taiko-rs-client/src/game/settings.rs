use piston::Key;
use tokio::sync::OnceCell;
use serde::{Serialize, Deserialize};
use taiko_rs_common::types::PlayMode;

use crate::sync::*;
use crate::Vector2;
use super::managers::NotificationManager;

const SETTINGS_FILE:&str = "settings.json";

lazy_static::lazy_static! {
    static ref SETTINGS: Arc<Mutex<Settings>> = Arc::new(Mutex::new(Settings::load()));

    pub static ref WINDOW_SIZE: OnceCell<Vector2> = OnceCell::new_with(Some(Settings::get_mut().window_size.into()));
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    // volume
    pub master_vol: f32,
    pub music_vol: f32,
    pub effect_vol: f32,

    // offset
    pub global_offset: f32,

    // login
    pub username: String,
    pub password: String,
    
    // osu login (for direct)
    pub osu_username: String,
    pub osu_password: String,
    
    pub standard_settings: StandardSettings,
    pub taiko_settings: TaikoSettings,
    pub catch_settings: CatchSettings,
    pub mania_settings: ManiaSettings,
    pub background_game_settings: BackgroundGameSettings,

    // window settings
    pub fps_target: u64,
    pub update_target: u64,
    pub window_size: [f64; 2],
    pub cursor_scale: f64,

    // bg
    pub background_dim: f32,
    /// should the game pause when focus is lost?
    pub pause_on_focus_lost: bool,

    pub cursor_color: String,


    // misc keybinds
    pub key_offset_up: Key,
    pub key_offset_down: Key,
}
impl Settings {
    fn load() -> Settings {
        let s = match std::fs::read_to_string(SETTINGS_FILE) {
            Ok(b) => match serde_json::from_str(&b) {
                Ok(settings) => settings,
                Err(e) => {
                    // println!("error reading settings.json, loading defaults");
                    NotificationManager::add_error_notification("Error reading settings.json\nLoading defaults", e);
                    Settings::default()
                }
            }
            Err(e) => {
                // println!("error reading settings.json, loading defaults");
                NotificationManager::add_error_notification("Error reading settings.json\nLoading defaults", e);
                Settings::default()
            }
        };
        // save after loading.
        // writes file if it doesnt exist, and writes new values from updates
        s.save();
        s
    }
    pub fn save(&self) {
        println!("Saving settings");
        let str = serde_json::to_string_pretty(self).unwrap();
        match std::fs::write(SETTINGS_FILE, str) {
            Ok(_) => println!("settings saved successfully"),
            Err(e) => NotificationManager::add_error_notification("Error saving settings", e),
        }
    }

    /// relatively slow, if you need a more performant get, use get_mut
    pub fn get() -> Settings {SETTINGS.lock().clone()}
    pub fn get_mut<'a>() -> MutexGuard<'a, Settings> {SETTINGS.lock()}

    pub fn window_size() -> Vector2 {*WINDOW_SIZE.get().unwrap()}

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
            global_offset: 0.0,

            // username
            username: "Guest".to_owned(),
            password: String::new(),

            // osu
            osu_username: String::new(),
            osu_password: String::new(),

            // mode settings
            standard_settings: StandardSettings{..Default::default()},
            taiko_settings: TaikoSettings {..Default::default()},
            catch_settings: CatchSettings {..Default::default()},
            mania_settings: ManiaSettings {..Default::default()},
            background_game_settings: BackgroundGameSettings {..Default::default()},

            // window settings
            fps_target: 144,
            update_target: 1000,
            window_size: [1000.0, 600.0],
            background_dim: 0.8,

            // other
            cursor_scale: 1.0,
            pause_on_focus_lost: true,

            cursor_color: "#ffff32".to_owned(),

            // keys
            key_offset_up: Key::Equals,
            key_offset_down: Key::Minus,
        }
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TaikoSettings {
    // sv
    pub static_sv: bool,
    pub sv_multiplier: f32,

    // keys
    pub left_kat: Key,
    pub left_don: Key,
    pub right_don: Key,
    pub right_kat: Key,

    pub ignore_mouse_buttons: bool
}
impl Default for TaikoSettings {
    fn default() -> Self {
        Self {
            // keys
            left_kat: Key::D,
            left_don: Key::F,
            right_don: Key::J,
            right_kat: Key::K,

            // sv
            static_sv: false,
            sv_multiplier: 1.0,
            
            ignore_mouse_buttons: false
        }
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ManiaSettings {
    // sv
    pub static_sv: bool,
    pub sv_multiplier: f32,

    /// col_count [col_num, 0 based]
    /// ie for 4k, key 2: mania_keys[4][1]
    pub keys: Vec<Vec<Key>>,
}
impl Default for ManiaSettings {
    fn default() -> Self {
        Self {
            // keys
            keys: vec![
                vec![Key::Space], // 1k
                vec![Key::F, Key::J], // 2k
                vec![Key::F, Key::Space, Key::J], // 3k
                vec![Key::D, Key::F, Key::J, Key::K], // 4k
                vec![Key::D, Key::F, Key::Space, Key::J, Key::K], // 5k
                vec![Key::S, Key::D, Key::F, Key::J, Key::K, Key::L], // 6k
                vec![Key::S, Key::D, Key::F, Key::Space, Key::J, Key::K, Key::L], // 7k
                vec![Key::A, Key::S, Key::D, Key::F, Key::J, Key::K, Key::L, Key::Semicolon], // 8k
                vec![Key::A, Key::S, Key::D, Key::F, Key::Space, Key::J, Key::K, Key::L, Key::Semicolon], // 9k
            ],

            // sv
            static_sv: false,
            sv_multiplier: 1.0,
        }
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct StandardSettings {
    // input
    pub left_key: Key,
    pub right_key: Key,
    pub ignore_mouse_buttons: bool,

    // playfield
    pub playfield_x_offset: f64,
    pub playfield_y_offset: f64,
    pub playfield_scale: f64,

    // display
    pub draw_follow_points: bool
}
impl StandardSettings {
    pub fn get_playfield(&self) -> (f64, Vector2) {
        (self.playfield_scale, Vector2::new(self.playfield_x_offset, self.playfield_y_offset))
    }
}
impl Default for StandardSettings {
    fn default() -> Self {
        Self {
            // keys
            left_key: Key::S,
            right_key: Key::D,
            ignore_mouse_buttons: false,

            playfield_x_offset: 0.0,
            playfield_y_offset: 0.0,
            playfield_scale: 0.8,

            draw_follow_points: true
        }
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CatchSettings {
    // keys
    pub left_key: Key,
    pub right_key: Key,
    pub dash_key: Key,
}
impl Default for CatchSettings {
    fn default() -> Self {
        Self {
            // keys
            left_key: Key::Left,
            right_key: Key::Right,
            dash_key: Key::LShift,
        }
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct BackgroundGameSettings {
    /// wether to have gameplay in the main menu bg or not
    pub enabled: bool,
    /// gameplay alpha multiplier
    pub opacity: f32,
    /// hitsound volume multiplier
    pub hitsound_volume: f32,
    /// what mode should be playing?
    pub mode: PlayMode,
}
impl Default for BackgroundGameSettings {
    fn default() -> Self {
        Self { 
            enabled: true,
            opacity: 0.5,
            hitsound_volume: 0.3,
            mode: PlayMode::Standard
        }
    }
}