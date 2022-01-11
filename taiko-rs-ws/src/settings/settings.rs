use crate::prelude::*;

const SETTINGS_FILE: &'static str = "./settings.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub port: u16,
    pub postgres: PostgresSettings,
}
impl Settings {
    /// try to load the settings file
    pub fn load() -> Self {
        // if the file doesnt exist, write the defaults and exit
        if !exists(SETTINGS_FILE) {
            Self::default().save();
            println!("[Settings] No settings found, generated defaults and exiting");
            std::process::exit(1);
        }

        // i hope this can be simplified somehow
        match std::fs::read_to_string(SETTINGS_FILE) {
            // file read okay, need to parse
            Ok(data) => match serde_json::from_str(&data) {
                // parse ok, return settings
                Ok(settings) => settings,

                // error parsing, probably user broke the json model
                Err(e) => {
                    // log the error
                    println!("[Settings] Error parsing settings.json: {}", e);

                    // close the program since we dont want to do anything if we cant read the settings file
                    std::process::exit(1);
                }
            },

            // there was an issue reading the file.
            Err(e) => {
                // log the error
                println!("[Settings] Error reading settings.json: {}", e);

                // close the program since we dont want to do anything if we cant read the settings file
                std::process::exit(1);
            }
        }
    }

    /// write current settings
    pub fn save(&self) {
        let contents = serde_json::to_string_pretty(self).unwrap();
        if let Err(e) = std::fs::write(SETTINGS_FILE, contents) {
            println!("[Settings] Error writing default settings.json: {}", e);
        }
        // else {
        //     println!("Creating default settings.json. Please edit the values and relaunch the bot.")
        // }
    }
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            port: 8080,
            postgres: PostgresSettings::new(),
        }
    }
}



